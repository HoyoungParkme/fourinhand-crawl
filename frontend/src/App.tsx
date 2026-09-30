// 화면 상태의 주인. 라우팅 없음 — 화면 주소가 하나다
import { useCallback, useEffect, useRef, useState } from 'react'
import {
  loadAnimals,
  openLink,
  queryAnimals,
  sameCondition,
  type AnimalView,
  type ApiError,
  type Condition,
  type ErrorCode,
  type LinkKind,
  type LoadResult,
} from './api/animals'
import AboutDialog from './pages/AboutDialog'
import DetailPanel, { type LinkFailure } from './pages/DetailPanel'
import ListPage, { type Phase, type Shown } from './pages/ListPage'
import { regionText } from './text/format'

/** 앱을 켤 때의 조건: 8.0kg 이상, 전체 기간, 전국, 무거운 순 */
const DEFAULT_CONDITION: Condition = { minWeightKg: 8, period: 'all', region: { kind: 'all' }, sort: 'weight' }

/** 열린 상세 — 연 때의 동물(목록이 바뀌어도 패널은 그대로)과 처음 보일 링크 실패 */
interface Detail {
  animal: AnimalView
  failure: LinkFailure | null
}

/** PC의 오늘 — 공고 상태와 기간은 조회할 때의 오늘로 정해진다 */
const today = () => new Date().toDateString()

export default function App() {
  const [phase, setPhase] = useState<Phase>('loading')
  const [received, setReceived] = useState<number | null>(null)
  const [failure, setFailure] = useState<ErrorCode | null>(null)
  const [fetched, setFetched] = useState<LoadResult | null>(null)
  const [condition, setCondition] = useState(DEFAULT_CONDITION)
  const [shown, setShown] = useState<Shown | null>(null)
  const [detail, setDetail] = useState<Detail | null>(null)
  const [aboutOpen, setAboutOpen] = useState(false)
  const [refreshing, setRefreshing] = useState(false)
  const [refreshError, setRefreshError] = useState<ErrorCode | null>(null)
  // 새로고침 뒤 사라져서 전국으로 돌린 지역
  const [regionResetFrom, setRegionResetFrom] = useState<string | null>(null)
  // 비동기 흐름이 「지금 조건」과 「받은 적이 있는지」를 읽는 자리
  const conditionRef = useRef(condition)
  const shownRef = useRef<Shown | null>(null)
  const fetchedRef = useRef<LoadResult | null>(null)
  const queriedOn = useRef('')
  const requestNo = useRef(0)
  const loading = useRef(false)
  // 상세를 닫으면 포커스를 돌려줄 카드
  const returnFocusTo = useRef<string | null>(null)

  /** 조회해서 그린다. 가장 마지막에 부른 조회의 결과만 그린다(API 3장) */
  const runQuery = useCallback(async (next: Condition) => {
    const no = ++requestNo.current
    const result = await queryAnimals(next)
    if (no !== requestNo.current) return
    queriedOn.current = today()
    // 고른 지역이 목록에서 사라졌으면 코어가 전국으로 걸렀다 — 지역 칸도 전국으로 돌리고 알린다
    const applied: Condition = result.regionReset ? { ...next, region: { kind: 'all' } } : next
    if (result.regionReset && next.region.kind !== 'all') setRegionResetFrom(regionText(next.region))
    conditionRef.current = applied
    setCondition(applied)
    shownRef.current = { condition: applied, result }
    setShown(shownRef.current)
  }, [])

  const changeCondition = useCallback(
    (next: Condition) => {
      // 이미 고른 칸을 다시 누른 것은 조건이 바뀐 것이 아니다 — 다시 조회하지도, 목록을 올리지도 않는다
      if (sameCondition(next, conditionRef.current)) return
      conditionRef.current = next
      setCondition(next)
      setRegionResetFrom(null)
      runQuery(next).catch((error: ApiError) => {
        // invalid-condition이면 마지막으로 올바르던 결과를 그대로 둔다. 나머지는 차례를 지키면 생기지 않는다
        console.warn('query_animals', error)
      })
    },
    [runQuery],
  )

  /**
   * 전부 받고 지금 조건으로 조회한다. 처음이면 받는 중 화면(6)을, 아니면 새로고침 버튼만 「받는 중…」으로 바꾸고
   * 목록과 조건은 그대로 쓰게 둔다. 새로고침이 실패하면 보던 목록과 받은 시각을 그대로 두고 띠(8)로 알린다.
   * 코어가 이미 받는 중이면 새로 받지 않고 그 받기의 결과를 함께 받는다(화면을 다시 불러왔을 때)
   */
  const refresh = useCallback(async () => {
    if (loading.current) return
    loading.current = true
    const first = fetchedRef.current === null
    if (first) {
      setPhase('loading')
      setReceived(null)
    } else {
      setRefreshing(true)
    }
    setRegionResetFrom(null)
    try {
      let loaded: LoadResult
      try {
        loaded = await loadAnimals((n) => {
          if (first) setReceived(n)
        })
      } catch (error) {
        const { code } = error as ApiError
        if (first) {
          setFailure(code)
          setPhase('failed')
        } else {
          setRefreshError(code)
        }
        return
      }
      fetchedRef.current = loaded
      setFetched(loaded)
      setRefreshError(null)
      // 받기는 성공했다 — 뒤따르는 조회가 실패해도 받기 실패로 알리지 않는다
      await runQuery(conditionRef.current).catch((error: ApiError) => console.warn('query_animals', error))
      setPhase('ready')
    } finally {
      loading.current = false
      setRefreshing(false)
    }
  }, [runQuery])

  // 켤 때 한 번만 받는다
  useEffect(() => {
    void refresh()
  }, [])

  // 켜 둔 채 날짜가 바뀌면 다시 조회한다 — 공고중·보호중과 기간은 조회할 때의 오늘로 정해진다
  useEffect(() => {
    const timer = window.setInterval(() => {
      if (!fetchedRef.current || queriedOn.current === today()) return
      runQuery(conditionRef.current).catch((error: ApiError) => console.warn('query_animals', error))
    }, 60_000)
    return () => window.clearInterval(timer)
  }, [runQuery])

  const findAnimal = (noticeNo: string) => shownRef.current?.result.animals.find((animal) => animal.noticeNo === noticeNo)

  const openDetail = useCallback((noticeNo: string) => {
    const animal = findAnimal(noticeNo)
    if (!animal) return
    returnFocusTo.current = noticeNo
    setDetail({ animal, failure: null })
  }, [])

  const closeDetail = useCallback(() => setDetail(null), [])

  // 상세를 닫으면 눌렀던 카드로 포커스를 돌려준다
  useEffect(() => {
    if (detail || !returnFocusTo.current) return
    const noticeNo = returnFocusTo.current
    returnFocusTo.current = null
    document.querySelector<HTMLElement>(`.card[data-notice-no="${CSS.escape(noticeNo)}"]`)?.focus()
  }, [detail])

  /** 링크를 연다. 카드에서 실패하면 알릴 자리가 없으므로 그 아이의 상세를 실패 상태로 연다 */
  const openLinkFor = useCallback(async (noticeNo: string, kind: LinkKind) => {
    try {
      await openLink(noticeNo, kind)
      setDetail((current) => (current?.animal.noticeNo === noticeNo ? { ...current, failure: null } : current))
    } catch (error) {
      const { code, url } = error as ApiError
      const linkFailure: LinkFailure | null =
        code === 'open-failed' ? { code, url: url ?? '' } : code === 'not-allowed' ? { code } : null
      if (!linkFailure) {
        // no-snapshot·not-found·no-source는 차례를 지키면 생기지 않는다
        console.warn('open_link', error)
        return
      }
      // 기다리는 동안 앱 정보를 열었더라도 실패 안내가 한 겹으로 보이게 닫는다
      setAboutOpen(false)
      setDetail((current) => {
        const animal = current?.animal.noticeNo === noticeNo ? current.animal : findAnimal(noticeNo)
        if (!animal) return current
        if (returnFocusTo.current === null) returnFocusTo.current = noticeNo
        return { animal, failure: linkFailure }
      })
    }
  }, [])

  // 앱 정보를 닫으면 i 버튼으로 포커스를 돌려준다
  const closeAbout = useCallback(() => {
    setAboutOpen(false)
    requestAnimationFrame(() => document.getElementById('about-button')?.focus())
  }, [])

  return (
    <div className="app">
      <ListPage
        phase={phase}
        received={received}
        failure={failure}
        fetched={fetched}
        condition={condition}
        shown={shown}
        refreshing={refreshing}
        refreshError={refreshError}
        regionResetFrom={regionResetFrom}
        blocked={detail !== null || aboutOpen}
        onRefresh={refresh}
        onChangeCondition={changeCondition}
        onOpenDetail={openDetail}
        onOpenLink={(noticeNo) => void openLinkFor(noticeNo, 'pawinhand')}
        onOpenAbout={() => setAboutOpen(true)}
      />
      {detail && (
        <DetailPanel
          key={detail.animal.noticeNo}
          animal={detail.animal}
          failure={detail.failure}
          onOpenLink={(kind) => void openLinkFor(detail.animal.noticeNo, kind)}
          onClose={closeDetail}
        />
      )}
      {aboutOpen && !detail && <AboutDialog onClose={closeAbout} />}
    </div>
  )
}
