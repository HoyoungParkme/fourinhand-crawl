# AGENTS.md — 에이전트용 안내

포인핸드 대형묘 찾기: 포인핸드에 보호 중인 고양이 가운데 몸무게가 기준 이상인 아이만 보여주는 Windows 앱(Tauri 2). 사람용 안내는 [README.md](README.md).

## 명세가 먼저

- 명세 원본은 `docs/specs/`(싱크독 프로젝트 `PAW`). 이 폴더는 싱크독이 커밋한다 — 로컬에서 고치지 않고 싱크독 MCP(`get_document` → `update_document`)로만 고친다
- 코드가 명세와 달라져야 하면 명세를 먼저 고친다
- 어디를 보면 되나: 함수의 입력·처리·예외는 `10-MS`, 파일 자리와 의존 방향은 `06-DOM/PAW-DOM-002`, 커맨드 모양은 `08-API`, 화면은 `07-UI`, 빌드·배포는 `05-INFRA`, 구현 순서는 `11-CODE`

## 구조와 규칙

| 자리 | 규칙 |
|---|---|
| `backend/src/domains/animal/models.rs` | 개념과 자기 값을 만드는 규칙(몸무게 해석·사진 주소·링크). 네트워크·시계·전역 상태를 쓰지 않는다 — 오늘 날짜도 인자로 받는다 |
| `backend/src/domains/animal/service.rs` | 받아오기·조회·링크 열기의 흐름. Tauri를 모른다 — 가짜 원천·가짜 열기로 테스트한다 |
| `backend/src/domains/animal/adapters/pawinhand.rs` | 포인핸드 칸 이름(`notify_number` 같은 것)은 이 파일 밖에 나오지 않는다 |
| `backend/src/domains/animal/commands.rs` | 입출력만. 판단하지 않는다 |
| `backend/src/core/error.rs` | 화면에 가는 오류 코드 목록은 여기 한 곳 |
| `frontend/src/api/animals.ts` | `invoke`를 부르는 유일한 파일 |
| `frontend/src/styles.css` | UI 토큰의 유일한 자리. 컴포넌트에 색·크기 값을 직접 쓰지 않는다 |
| `frontend/src/text/format.ts` | 화면 글 규칙(무게·나이·날짜·받침·실패 문구) |

## 명령 (저장소 루트에서)

- 개발 실행: `pwsh scripts/dev.ps1`
- 설치 파일: `pwsh scripts/build.ps1`
- 코어 테스트: `cd backend; cargo test` — 실데이터 점검은 `cargo test -- --ignored`
- 화면 타입 검사: `cd frontend; npx tsc`

## 버전·브랜치·커밋

- 버전은 `backend/tauri.conf.json`의 `version` 한 곳. 태그 `v{버전}`을 올리면 `.github/workflows/release.yml`이 설치 파일을 Releases에 올린다
- `main` + 작업 브랜치(`feat/…` · `fix/…` · `exp/…`), 작업 브랜치는 `main`에 squash 머지. force push 하지 않는다
- 커밋 메시지와 PR 본문에 에이전트 표시 줄(`Co-Authored-By: …`, `Claude-Session: …`, 「Generated with …」)을 넣지 않는다(싱크독 규약 1.10)
- 앱 식별자 `io.github.hoyoungparkme.pawinhandbigcat`은 바꾸지 않는다 — 바꾸면 덮어 설치가 안 된다(INFRA C4)
