//! 받아온 목록 하나와 「받는 중」 표시를 든다

use std::sync::{Arc, PoisonError, RwLock};

use tokio::sync::watch;

use super::models::{FetchError, Snapshot};

/// 받기 한 번의 결과
pub type LoadResult = Result<Arc<Snapshot>, FetchError>;

#[derive(Debug, Default)]
struct LoadState {
    loading: bool,
    /// 끝난 받기의 수 — 기다리는 쪽이 「내가 온 뒤에 끝난 받기」를 알아보는 표
    finished: u64,
    last: Option<LoadResult>,
}

#[derive(Debug)]
pub struct AnimalStore {
    snapshot: RwLock<Option<Arc<Snapshot>>>,
    load: watch::Sender<LoadState>,
}

impl Default for AnimalStore {
    fn default() -> Self {
        Self { snapshot: RwLock::default(), load: watch::Sender::new(LoadState::default()) }
    }
}

/// 받기를 맡을 차례인지, 이미 누가 받는 중이라 기다려야 하는지
pub enum LoadTurn<'a> {
    Mine(LoadGuard<'a>),
    /// 이 수보다 많은 받기가 끝나면 그 결과를 쓴다
    Wait(u64),
}

impl AnimalStore {
    /// 지금 스냅숏. `Arc`만 복사한다 — 목록은 복사하지 않는다
    pub fn current(&self) -> Option<Arc<Snapshot>> {
        self.snapshot.read().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// 스냅숏을 통째로 간다. 읽는 쪽은 옛것이나 새것 하나만 본다 — 반쯤 바뀐 목록은 없다
    pub fn replace(&self, snapshot: Snapshot) -> Arc<Snapshot> {
        let snapshot = Arc::new(snapshot);
        *self.snapshot.write().unwrap_or_else(PoisonError::into_inner) = Some(Arc::clone(&snapshot));
        snapshot
    }

    /// 「받는 중」을 잡는다. 이미 누가 받는 중이면 새로 받지 않고 기다릴 표를 준다
    pub fn try_begin_load(&self) -> LoadTurn<'_> {
        let mut waiting = None;
        self.load.send_if_modified(|state| {
            if state.loading {
                waiting = Some(state.finished);
                false
            } else {
                state.loading = true;
                true
            }
        });
        match waiting {
            Some(finished) => LoadTurn::Wait(finished),
            None => LoadTurn::Mine(LoadGuard { store: self, finished: false }),
        }
    }

    /// 표를 받은 뒤에 끝난 받기의 결과를 기다린다 — 받는 중에 들어온 부르기도 같은 결과를 받는다
    pub async fn wait_for_load(&self, after: u64) -> LoadResult {
        let mut rx = self.load.subscribe();
        let state = rx
            .wait_for(|state| state.finished > after)
            .await
            .expect("보관소가 살아 있는 동안 보내는 쪽도 살아 있다");
        state.last.clone().expect("끝난 받기에는 결과가 있다")
    }

    fn finish_load(&self, result: LoadResult) {
        self.load.send_modify(|state| {
            state.loading = false;
            state.finished += 1;
            state.last = Some(result);
        });
    }
}

pub struct LoadGuard<'a> {
    store: &'a AnimalStore,
    finished: bool,
}

impl LoadGuard<'_> {
    /// 결과를 알리고 「받는 중」을 푼다
    pub fn finish(mut self, result: &LoadResult) {
        self.finished = true;
        self.store.finish_load(result.clone());
    }
}

impl Drop for LoadGuard<'_> {
    /// 결과를 알리지 못하고 떨어져도(패닉) 기다리는 쪽이 멈추지 않게 실패로 알린다
    fn drop(&mut self) {
        if !self.finished {
            self.store.finish_load(Err(FetchError::ConnectionFailed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    fn snapshot(fetched_at: &str) -> Snapshot {
        Snapshot::new(vec![], DateTime::parse_from_rfc3339(fetched_at).unwrap())
    }

    #[test]
    fn replace_leaves_earlier_readers_on_the_old_snapshot() {
        let store = AnimalStore::default();
        assert!(store.current().is_none());
        store.replace(snapshot("2026-09-30T14:02:11+09:00"));
        let before = store.current().unwrap();
        store.replace(snapshot("2026-09-30T15:00:00+09:00"));
        assert_eq!(before.fetched_at.to_rfc3339(), "2026-09-30T14:02:11+09:00");
        assert_eq!(store.current().unwrap().fetched_at.to_rfc3339(), "2026-09-30T15:00:00+09:00");
    }

    #[tokio::test]
    async fn a_running_load_hands_its_result_to_waiters() {
        let store = AnimalStore::default();
        let LoadTurn::Mine(guard) = store.try_begin_load() else { panic!("처음 부르면 받기를 맡는다") };
        let LoadTurn::Wait(after) = store.try_begin_load() else { panic!("받는 중에는 기다린다") };
        let loaded = store.replace(snapshot("2026-09-30T14:02:11+09:00"));
        let (waited, ()) = tokio::join!(store.wait_for_load(after), async { guard.finish(&Ok(loaded.clone())) });
        assert!(Arc::ptr_eq(&waited.unwrap(), &loaded));
        assert!(matches!(store.try_begin_load(), LoadTurn::Mine(_)), "끝난 뒤에는 다시 맡는다");
    }

    #[tokio::test]
    async fn a_dropped_load_does_not_leave_waiters_hanging() {
        let store = AnimalStore::default();
        let LoadTurn::Mine(guard) = store.try_begin_load() else { panic!("처음 부르면 받기를 맡는다") };
        let LoadTurn::Wait(after) = store.try_begin_load() else { panic!("받는 중에는 기다린다") };
        drop(guard);
        assert_eq!(store.wait_for_load(after).await.unwrap_err(), FetchError::ConnectionFailed);
    }
}
