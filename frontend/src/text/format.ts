// 화면 글 만들기 — UI 문서 「보여주는 글」 규칙
import type { AnimalView, ErrorCode, Period, RegionFilter } from '../api/animals'

const count = new Intl.NumberFormat('ko-KR')

/** 마릿수: 4225 → 4,225 */
export function formatCount(n: number): string {
  return count.format(n)
}

/** 무게는 저울 표시처럼 늘 소수 한 자리: 15 → 15.0 */
export function formatKg(kg: number): string {
  return kg.toFixed(1)
}

/** 2017(년생) → 2017년생, 2026(60일미만)(년생) → 2026년생 (60일 미만). 두 모양이 아니면 받은 글자 그대로 */
export function formatAge(raw: string | null): string {
  if (!raw) return ''
  const born = /^(\d{4})\(년생\)$/.exec(raw)
  if (born) return `${born[1]}년생`
  const young = /^(\d{4})\((\d+)일미만\)\(년생\)$/.exec(raw)
  if (young) return `${young[1]}년생 (${young[2]}일 미만)`
  return raw
}

export function formatSex(code: AnimalView['sex']): string {
  if (code === 'M') return '수컷'
  if (code === 'F') return '암컷'
  if (code === 'Q') return '성별 미상'
  return ''
}

export function formatNeutered(code: AnimalView['neutered']): string {
  if (code === 'Y') return '했음'
  if (code === 'N') return '안 했음'
  if (code === 'U') return '미확인'
  return ''
}

/** 카드의 「2017년생 수컷」. 나이가 비면 성별만 */
export function formatWho(animal: AnimalView): string {
  return [formatAge(animal.age), formatSex(animal.sex)].filter(Boolean).join(' ')
}

/** 카드의 「화성시민동물보호센터, 경기도 화성시」. 시·도가 비면 시·군·구만 */
export function formatWhere(animal: AnimalView): string {
  const { sido, sigungu } = animal.region
  // 제주·세종처럼 시·군·구 칸에 시·도 이름이 다시 오면 한 번만 쓴다
  const place = [sido, sigungu === sido ? null : sigungu].filter(Boolean).join(' ')
  return [animal.shelter.name, place].filter(Boolean).join(', ')
}

function dotted(isoDate: string): string {
  return isoDate.replaceAll('-', '.')
}

/** short(카드)면 같은 해의 끝 날짜는 월.일만: 2026.08.30 ~ 08.30. 상세는 늘 연도까지 */
export function formatPeriod(start: string, end: string, short: boolean): string {
  const sameYear = start.slice(0, 4) === end.slice(0, 4)
  return `${dotted(start)} ~ ${short && sameYear ? dotted(end.slice(5)) : dotted(end)}`
}

/** 받은 시각: 오늘이면 「오늘 14:02」, 아니면 「9월 29일 14:02」 */
export function formatFetchedAt(iso: string, now: Date = new Date()): string {
  const at = new Date(iso)
  const time = `${String(at.getHours()).padStart(2, '0')}:${String(at.getMinutes()).padStart(2, '0')}`
  const today = at.getFullYear() === now.getFullYear() && at.getMonth() === now.getMonth() && at.getDate() === now.getDate()
  return today ? `오늘 ${time}` : `${at.getMonth() + 1}월 ${at.getDate()}일 ${time}`
}

/** 끝 글자에 받침이 있으면 「이에요」, 없으면 「예요」 */
export function withCopula(word: string): string {
  const last = word.charCodeAt(word.length - 1)
  const isHangul = last >= 0xac00 && last <= 0xd7a3
  return isHangul && (last - 0xac00) % 28 !== 0 ? `${word}이에요` : `${word}예요`
}

export function periodText(period: Period): string {
  if (period === '1y') return '최근 1년'
  if (period === '6m') return '최근 6개월'
  if (period === '3m') return '최근 3개월'
  return '전체 기간'
}

export function regionText(region: RegionFilter): string {
  if (region.kind === 'sido') return region.name
  if (region.kind === 'unknown') return '지역 미상'
  return '전국'
}

/** 받아오기 실패의 제목과 이유 문장(UI-1 규칙의 표) */
export function failureText(code: ErrorCode): { title: string; reason: string } {
  switch (code) {
    case 'connection-failed':
      return { title: '포인핸드에 연결하지 못했어요', reason: '인터넷에 연결되어 있는지 확인한 뒤 다시 시도하세요.' }
    case 'timeout':
      return { title: '포인핸드에 연결하지 못했어요', reason: '포인핸드가 15초 동안 응답하지 않았어요. 잠시 뒤 다시 시도하세요.' }
    case 'too-many-pages':
      return { title: '포인핸드 목록을 읽지 못했어요', reason: '목록이 비정상적으로 길어 받기를 멈췄어요. 새 버전이 나왔는지 확인하세요.' }
    default:
      return { title: '포인핸드 목록을 읽지 못했어요', reason: '포인핸드가 예상과 다른 형식으로 답했어요. 새 버전이 나왔는지 확인하세요.' }
  }
}
