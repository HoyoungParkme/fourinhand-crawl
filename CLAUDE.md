# 포인핸드 대형묘 찾기 (fourinhand-crawl)

포인핸드에 보호 중인 고양이 가운데 몸무게가 기준(기본 8kg) 이상인 아이만 보여주는 Windows 데스크톱 앱(Tauri). 사람용 안내는 README.md, 에이전트용 안내는 AGENTS.md.

## 명세

- 명세 원본은 `docs/specs/`(싱크독 프로젝트 `PAW`). 이 폴더는 싱크독이 커밋한다 — 로컬에서 직접 고치지 않고 싱크독 MCP(`get_document` → `update_document`)로만 고친다
- 코드가 명세와 달라져야 하면 명세를 먼저 고친다

## 브랜치와 커밋

- 브랜치 전략: **운영 배포 없음** — `main` + 작업 브랜치(`feat/…` · `fix/…` · `exp/…`). 작업 브랜치는 `main`에 squash 머지
- 배포는 브랜치가 아니라 태그다: `v{버전}` 태그를 올리면 GitHub Actions가 설치 파일을 만들어 Releases에 올린다
- 커밋 메시지와 PR 본문에 에이전트 표시 줄(`Co-Authored-By: Claude …`, `Claude-Session: …`, 「Generated with Claude Code」)을 넣지 않는다 — 싱크독 규약 1.10(사용자 결정)
