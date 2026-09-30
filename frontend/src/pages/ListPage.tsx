// UI-1 목록
import { Fragment, useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from 'react'
import { sameRegion } from '../api/animals'
import type { AnimalView, Condition, ErrorCode, LoadResult, Period, QueryResult, RegionFilter, SortKey } from '../api/animals'
import PhotoBox from '../components/PhotoBox'
import StatusBadge from '../components/StatusBadge'
import WeightTag from '../components/WeightTag'
import {
  failureText,
  formatCount,
  formatFetchedAt,
  formatKg,
  formatPeriod,
  formatWhere,
  formatWho,
  periodText,
  regionText,
  withCopula,
} from '../text/format'

export type Phase = 'loading' | 'failed' | 'ready'

/** 그려진 목록과 그 목록을 거른 조건 */
export interface Shown {
  condition: Condition
  result: QueryResult
}

interface Props {
  phase: Phase
  /** 처음 받는 동안 지금까지 받은 마릿수 */
  received: number | null
  /** 처음 받기가 실패한 까닭 */
  failure: ErrorCode | null
  fetched: LoadResult | null
  condition: Condition
  shown: Shown | null
  /** 목록이 있는 채로 새로 받는 중 */
  refreshing: boolean
  /** 새로고침이 실패한 까닭 — 목록과 받은 시각은 그대로다 */
  refreshError: ErrorCode | null
  /** 새로고침 뒤 사라져서 전국으로 돌린 지역 */
  regionResetFrom: string | null
  /** 상세 패널·대화상자가 떠 있는 동안 목록과 윗줄은 누를 수 없다 */
  blocked: boolean
  onRefresh: () => void
  onChangeCondition: (next: Condition) => void
  onOpenDetail: (noticeNo: string) => void
  onOpenLink: (noticeNo: string) => void
  onOpenAbout: () => void
}

export default function ListPage({
  phase,
  received,
  failure,
  fetched,
  condition,
  shown,
  refreshing,
  refreshError,
  regionResetFrom,
  blocked,
  onRefresh,
  onChangeCondition,
  onOpenDetail,
  onOpenLink,
  onOpenAbout,
}: Props) {
  const ready = phase === 'ready'
  const busy = refreshing || phase === 'loading'
  const mainRef = useRef<HTMLElement>(null)
  const change = (patch: Partial<Condition>) => onChangeCondition({ ...condition, ...patch })

  // 조건을 바꾸면 결과 목록은 맨 위로 올라간다. 새로고침은 조건이 그대로라 보던 자리를 지킨다(패널 뒤 목록도)
  useEffect(() => {
    mainRef.current?.scrollTo({ top: 0 })
  }, [shown?.condition])

  return (
    <>
      <header className="top" inert={blocked}>
        <h1>포인핸드 대형묘 찾기</h1>
        <span className="got">
          {fetched
            ? `${formatFetchedAt(fetched.fetchedAt)}, 보호 중인 고양이 ${formatCount(fetched.total)}마리를 받았어요`
            : '아직 받은 목록이 없어요'}
        </span>
        <button type="button" className="btn" disabled={busy} onClick={onRefresh}>
          {busy ? '받는 중…' : '새로고침'}
        </button>
        <button type="button" className="btn info-btn" id="about-button" aria-label="앱 정보" onClick={onOpenAbout}>
          i
        </button>
      </header>
      <aside className={ready ? 'side' : 'side dimmed'} inert={!ready || blocked}>
        <section>
          <h2>몸무게 기준</h2>
          <WeightFilter kg={condition.minWeightKg} onChange={(minWeightKg) => change({ minWeightKg })} />
        </section>
        <section>
          <h2>등록 기간</h2>
          <PeriodFilter period={condition.period} onChange={(period) => change({ period })} />
        </section>
        {shown && (
          <section>
            <h2>지역</h2>
            <RegionList result={shown.result} region={condition.region} onChange={(region) => change({ region })} />
          </section>
        )}
        <p className="note">포인핸드 공식 앱이 아닙니다. 포인핸드에 공개된 공고를 모아 보여줍니다.</p>
      </aside>
      <main className="main" ref={mainRef} inert={blocked}>
        {ready && regionResetFrom && (
          <div className="banner ok" role="status">
            <p>{regionResetFrom}에 보호 중인 아이가 없어 지역을 전국으로 바꿨어요</p>
          </div>
        )}
        {ready && refreshError && fetched && (
          // 8 새로고침 실패 — 보던 목록과 받은 시각은 그대로 둔다
          <div className="banner err" role="alert">
            <p>
              <b>새로 받지 못했어요.</b> {failureText(refreshError).reason} 지금 보이는 목록은{' '}
              {formatFetchedAt(fetched.fetchedAt)}에 받은 거예요.
            </p>
            <button type="button" className="btn" disabled={refreshing} onClick={onRefresh}>
              다시 시도
            </button>
          </div>
        )}
        {phase === 'loading' && <Loading received={received} />}
        {phase === 'failed' && <Failed code={failure ?? 'bad-format'} onRetry={onRefresh} />}
        {ready && shown && (
          <Results
            shown={shown}
            fetchedAt={fetched?.fetchedAt ?? ''}
            sort={condition.sort}
            onChangeSort={(sort) => change({ sort })}
            onOpenDetail={onOpenDetail}
            onOpenLink={onOpenLink}
          />
        )}
      </main>
    </>
  )
}

const RULER_MAX = 20

/** 칸의 글자 → 0.1 단위 kg. 10진수가 아니거나(1e3·0x10 같은 모양 포함) 0보다 작으면 없다 */
function parseKg(text: string): number | null {
  const s = text.trim().replace(',', '.')
  if (!/^(\d+\.?\d*|\.\d+)$/.test(s)) return null
  const kg = Math.round(Number(s) * 10) / 10
  return Number.isFinite(kg) ? kg : null
}

/** 2.1 몸무게 기준 칸과 2.2 눈금자 — 같은 값이다 */
function WeightFilter({ kg, onChange }: { kg: number; onChange: (kg: number) => void }) {
  const [text, setText] = useState(formatKg(kg))
  const [bad, setBad] = useState(false)
  // 눈금자를 끄는 동안의 값 — 칸과 핀만 따라오고, 놓을 때 한 번 조회한다
  const [dragging, setDragging] = useState<number | null>(null)
  const timer = useRef<number | undefined>(undefined)
  // 입력한 뒤 아직 조회하지 않은 글자가 있는지 — blur는 이때만 조회한다(눈금자로 바꾼 값을 되돌리지 않게)
  const dirty = useRef(false)

  // 눈금자처럼 밖에서 기준이 바뀌면 칸을 맞춘다. 치고 있는 글자가 같은 값이면 그대로 둔다
  useEffect(() => {
    setText((current) => (parseKg(current) === kg ? current : formatKg(kg)))
    setBad(false)
    dirty.current = false
  }, [kg])

  useEffect(() => () => window.clearTimeout(timer.current), [])

  const commit = (value: string, reformat: boolean) => {
    window.clearTimeout(timer.current)
    dirty.current = false
    const next = parseKg(value)
    setBad(next === null)
    if (next === null) return
    if (reformat) setText(formatKg(next))
    if (next !== kg) onChange(next)
  }

  const type = (value: string) => {
    setText(value)
    dirty.current = true
    window.clearTimeout(timer.current)
    // 입력이 멈추고 150ms 뒤에만 거른다
    timer.current = window.setTimeout(() => commit(value, false), 150)
  }

  // 칸에 보이는 글자가 올바르면 경고를 띄우지 않는다
  const invalid = bad && parseKg(text) === null

  return (
    <>
      <div className="wt">
        <input
          className={invalid ? 'bad' : undefined}
          value={dragging === null ? text : formatKg(dragging)}
          inputMode="decimal"
          aria-label="몸무게 기준(kg)"
          aria-invalid={invalid}
          onChange={(e) => type(e.target.value)}
          onBlur={() => {
            if (dirty.current) commit(text, true)
            else if (!invalid) setText(formatKg(kg))
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit(text, true)
          }}
        />
        <span>kg 이상</span>
      </div>
      <Ruler
        kg={dragging ?? kg}
        onDrag={setDragging}
        onChange={(next) => {
          setDragging(null)
          if (next !== kg) onChange(next)
        }}
      />
      {invalid && (
        <p className="hint" role="alert">
          0 이상의 숫자를 넣으세요. 목록은 {formatKg(kg)}kg 기준 그대로예요.
        </p>
      )}
    </>
  )
}

interface RulerProps {
  kg: number
  /** 끄는 동안 — 조회하지 않는다 */
  onDrag: (kg: number) => void
  /** 놓았거나 키보드로 바꿨을 때 — 조회한다 */
  onChange: (kg: number) => void
}

/** 2.2 눈금자. 0~20kg만 보이고, 그보다 큰 기준이면 핀이 오른쪽 끝에 붙는다 */
function Ruler({ kg, onDrag, onChange }: RulerProps) {
  const dragging = useRef(false)
  const at = `${(Math.min(Math.max(kg, 0), RULER_MAX) / RULER_MAX) * 100}%`
  const set = (value: number, max = RULER_MAX) => onChange(Math.round(Math.min(Math.max(value, 0), max) * 10) / 10)
  const valueAt = (e: PointerEvent<HTMLDivElement>) => {
    const box = e.currentTarget.getBoundingClientRect()
    const ratio = Math.min(Math.max((e.clientX - box.left) / box.width, 0), 1)
    return Math.round(ratio * RULER_MAX * 10) / 10
  }
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    const steps: Record<string, number> = { ArrowLeft: -0.1, ArrowDown: -0.1, ArrowRight: 0.1, ArrowUp: 0.1, PageDown: -1, PageUp: 1 }
    const max = Math.max(RULER_MAX, kg)
    if (e.key in steps) set(kg + steps[e.key], max)
    else if (e.key === 'Home') set(0)
    else if (e.key === 'End') set(RULER_MAX)
    else return
    e.preventDefault()
  }
  return (
    <div
      className="ruler"
      role="slider"
      tabIndex={0}
      aria-label="몸무게 기준 눈금자"
      aria-valuemin={0}
      aria-valuemax={RULER_MAX}
      aria-valuenow={Math.min(kg, RULER_MAX)}
      aria-valuetext={`${formatKg(kg)}kg 이상`}
      onPointerDown={(e) => {
        if (e.button !== 0) return
        // 칸의 blur가 눈금자 값보다 먼저 끝나게 포커스부터 가져온다
        e.currentTarget.focus()
        e.currentTarget.setPointerCapture(e.pointerId)
        dragging.current = true
        onDrag(valueAt(e))
      }}
      onPointerMove={(e) => {
        if (dragging.current) onDrag(valueAt(e))
      }}
      onPointerUp={(e) => {
        if (!dragging.current) return
        dragging.current = false
        onChange(valueAt(e))
      }}
      onPointerCancel={() => {
        if (!dragging.current) return
        dragging.current = false
        onChange(kg)
      }}
      onKeyDown={onKeyDown}
    >
      <div className="ticks" />
      <div className="over" style={{ left: at }} />
      {[0, 5, 10, 15, 20].map((mark) => (
        <Fragment key={mark}>
          <div className="major" style={{ left: mark === RULER_MAX ? 'calc(100% - 1px)' : `${(mark / RULER_MAX) * 100}%` }} />
          <span
            className={mark === 0 ? 'lab first' : mark === RULER_MAX ? 'lab last' : 'lab'}
            style={{ left: `${(mark / RULER_MAX) * 100}%` }}
          >
            {mark}
          </span>
        </Fragment>
      ))}
      <div className="pin" style={{ left: at }} />
    </div>
  )
}

const PERIODS: [Period, string][] = [
  ['all', '전체'],
  ['1y', '1년'],
  ['6m', '6개월'],
  ['3m', '3개월'],
]

/** 2.3 등록 기간 */
function PeriodFilter({ period, onChange }: { period: Period; onChange: (period: Period) => void }) {
  return (
    <div className="seg" role="radiogroup" aria-label="등록 기간">
      {PERIODS.map(([value, label]) => (
        <button
          key={value}
          type="button"
          role="radio"
          aria-checked={value === period}
          className={value === period ? 'on' : undefined}
          onClick={() => onChange(value)}
        >
          {label}
        </button>
      ))}
    </div>
  )
}

interface RegionRow {
  key: string
  label: string
  count: number
  filter: RegionFilter
  zero: boolean
}

/** 전국 → 마릿수 많은 순(같으면 가나다) → 0마리 시·도(가나다) → 지역 미상 */
function regionRows(result: QueryResult): RegionRow[] {
  const byName = (a: RegionRow, b: RegionRow) => a.label.localeCompare(b.label, 'ko')
  const sidos: RegionRow[] = result.regionCounts.flatMap((rc) =>
    rc.kind === 'sido'
      ? [{ key: `sido:${rc.name}`, label: rc.name, count: rc.count, filter: { kind: 'sido', name: rc.name }, zero: rc.count === 0 }]
      : [],
  )
  const unknown = result.regionCounts.find((rc) => rc.kind === 'unknown')
  return [
    { key: 'all', label: '전국', count: result.allCount, filter: { kind: 'all' }, zero: false },
    ...sidos.filter((row) => !row.zero).sort((a, b) => b.count - a.count || byName(a, b)),
    ...sidos.filter((row) => row.zero).sort(byName),
    ...(unknown
      ? [{ key: 'unknown', label: '지역 미상', count: unknown.count, filter: { kind: 'unknown' } as const, zero: unknown.count === 0 }]
      : []),
  ]
}

/** 2.4 지역 — 숫자는 지역 조건만 뺀 나머지 조건으로 센 수 */
function RegionList({ result, region, onChange }: { result: QueryResult; region: RegionFilter; onChange: (region: RegionFilter) => void }) {
  return (
    <ul className="regions" aria-label="지역">
      {regionRows(result).map((row) => {
        const on = sameRegion(row.filter, region)
        return (
          <li
            key={row.key}
            role="button"
            tabIndex={0}
            aria-pressed={on}
            className={on ? 'on' : row.zero ? 'zero' : undefined}
            onClick={() => onChange(row.filter)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault()
                onChange(row.filter)
              }
            }}
          >
            <span>{row.label}</span>
            <b>{formatCount(row.count)}</b>
          </li>
        )
      })}
    </ul>
  )
}

/** 6 받는 중 */
function Loading({ received }: { received: number | null }) {
  return (
    <div className="center">
      <p className="t">포인핸드에서 보호 중인 고양이를 받고 있어요</p>
      <div className="bar" role="progressbar" aria-label="받는 중" />
      <p className="d" aria-live="polite">
        {received === null ? ' ' : `${formatCount(received)}마리 받음`}
      </p>
    </div>
  )
}

/** 7 받아오기 실패 */
function Failed({ code, onRetry }: { code: ErrorCode; onRetry: () => void }) {
  const { title, reason } = failureText(code)
  return (
    <div className="center" role="alert">
      <p className="t">{title}</p>
      <p className="d">{reason}</p>
      <button type="button" className="btn primary" onClick={onRetry}>
        다시 시도
      </button>
    </div>
  )
}

interface ResultsProps {
  shown: Shown
  /** 받은 시각 — 새로 받으면 못 불러왔던 사진을 다시 불러온다 */
  fetchedAt: string
  sort: SortKey
  onChangeSort: (sort: SortKey) => void
  onOpenDetail: (noticeNo: string) => void
  onOpenLink: (noticeNo: string) => void
}

/** 카드는 한 번에 이만큼씩 그린다 — 기준을 낮추면 수천 마리가 걸려 한꺼번에 그리면 창이 멈춘다 */
const CARDS_PER_STEP = 60

function Results({ shown, fetchedAt, sort, onChangeSort, onOpenDetail, onOpenLink }: ResultsProps) {
  const { condition, result } = shown
  const [limit, setLimit] = useState(CARDS_PER_STEP)
  const more = useRef<HTMLDivElement>(null)
  const hasMore = result.animals.length > limit

  // 조건이 바뀌면 첫 묶음부터 그린다. 새로고침은 조건이 그대로라 보던 만큼 둔다
  useEffect(() => setLimit(CARDS_PER_STEP), [condition])

  // 목록 끝이 가까워지면 다음 묶음을 그린다
  useEffect(() => {
    const sentinel = more.current
    if (!sentinel) return
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) setLimit((n) => n + CARDS_PER_STEP)
      },
      { rootMargin: '800px 0px' },
    )
    observer.observe(sentinel)
    return () => observer.disconnect()
  }, [hasMore, limit])

  if (result.count === 0) {
    // 9 조건에 맞는 아이가 없음
    return (
      <div className="center">
        <p className="t">조건에 맞는 아이가 없어요</p>
        <p className="d">
          지금 조건은 {formatKg(condition.minWeightKg)}kg 이상, {periodText(condition.period)},{' '}
          {withCopula(regionText(condition.region))}. 몸무게 기준을 낮추거나 기간이나 지역을 넓혀 보세요.
        </p>
      </div>
    )
  }
  return (
    <>
      <div className="head">
        <p className="count">{formatCount(result.count)}마리</p>
        <label className="sort">
          정렬
          <select value={sort} onChange={(e) => onChangeSort(e.target.value as SortKey)}>
            <option value="weight">무거운 순</option>
            <option value="registered">최근 등록 순</option>
          </select>
        </label>
      </div>
      <div className="grid">
        {result.animals.slice(0, limit).map((animal) => (
          <Card
            key={animal.noticeNo}
            animal={animal}
            fetchedAt={fetchedAt}
            onOpen={() => onOpenDetail(animal.noticeNo)}
            onOpenLink={() => onOpenLink(animal.noticeNo)}
          />
        ))}
      </div>
      {hasMore && <div ref={more} className="more" aria-hidden="true" />}
    </>
  )
}

/** 5 카드. 어디를 눌러도 상세가 열리고, 5.6만은 바로 기본 브라우저로 간다 */
interface CardProps {
  animal: AnimalView
  fetchedAt: string
  onOpen: () => void
  onOpenLink: () => void
}

function Card({ animal, fetchedAt, onOpen, onOpenLink }: CardProps) {
  return (
    <article
      className="card"
      tabIndex={0}
      data-notice-no={animal.noticeNo}
      onClick={onOpen}
      onKeyDown={(e) => {
        // preventDefault — 같은 Enter가 열린 패널의 닫기 버튼을 누르지 않게
        if (e.key === 'Enter' && e.target === e.currentTarget) {
          e.preventDefault()
          onOpen()
        }
      }}
    >
      <PhotoBox key={fetchedAt} url={animal.photos[0]}>
        <StatusBadge status={animal.notice.status} />
        <WeightTag kg={animal.weight.kg} />
      </PhotoBox>
      <div className="body">
        <p className="who">{formatWho(animal)}</p>
        <p className="where">{formatWhere(animal)}</p>
        <p className="when">공고 {formatPeriod(animal.notice.start, animal.notice.end, true)}</p>
        <button
          type="button"
          className="go"
          onClick={(e) => {
            e.stopPropagation()
            onOpenLink()
          }}
        >
          포인핸드에서 보기
        </button>
      </div>
    </article>
  )
}
