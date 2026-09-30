// UI-2 상세 패널
import { useEffect, useRef, useState } from 'react'
import type { AnimalView, LinkKind } from '../api/animals'
import PhotoBox from '../components/PhotoBox'
import StatusBadge from '../components/StatusBadge'
import WeightTag from '../components/WeightTag'
import { formatAge, formatNeutered, formatPeriod, formatSex } from '../text/format'

/** 링크를 열지 못한 까닭. 브라우저를 못 열었으면 열려던 주소를 보여준다 */
export type LinkFailure = { code: 'open-failed'; url: string } | { code: 'not-allowed' }

interface Props {
  animal: AnimalView
  failure: LinkFailure | null
  onOpenLink: (kind: LinkKind) => void
  onClose: () => void
}

export default function DetailPanel({ animal, failure, onOpenLink, onClose }: Props) {
  const closeRef = useRef<HTMLButtonElement>(null)

  // 열리면 닫기에 포커스
  useEffect(() => {
    closeRef.current?.focus()
  }, [])

  // Esc로 닫는다
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose])

  return (
    <>
      <div className="shade" onClick={onClose} />
      <aside className="panel" role="dialog" aria-modal="true" aria-label="보호 동물 상세">
        <header className="p-head">
          <StatusBadge status={animal.notice.status} />
          <span className="no">{animal.noticeNo}</span>
          <button ref={closeRef} type="button" className="btn" onClick={onClose}>
            닫기
          </button>
        </header>
        <div className="p-body">
          <Gallery photos={animal.photos.slice(0, 3)} />
          <div className="weight">
            <WeightTag kg={animal.weight.kg} />
            <span className="raw">원래 값 {animal.weight.raw}</span>
          </div>
          <dl className="facts">
            <Fact label="나이" value={formatAge(animal.age)} />
            <Fact label="성별" value={formatSex(animal.sex)} />
            <Fact label="중성화" value={formatNeutered(animal.neutered)} />
            <Fact label="품종" value={animal.breed} />
            <Fact label="털색" value={animal.color} />
            <Fact label="특징" value={animal.feature} />
            <Fact label="발견 장소" value={animal.foundAt} />
            <Fact label="공고 기간" value={formatPeriod(animal.notice.start, animal.notice.end, false)} />
          </dl>
          <h3 className="facts-h">보호소</h3>
          <dl className="facts">
            <Fact label="이름" value={animal.shelter.name} />
            <Fact label="주소" value={animal.shelter.address} />
            <Fact label="전화" value={animal.shelter.tel} />
          </dl>
          <h3 className="facts-h">관할 기관</h3>
          <dl className="facts">
            <Fact label="이름" value={animal.office.name} />
            <Fact label="전화" value={animal.office.tel} />
          </dl>
        </div>
        <footer className="p-foot">
          {failure && <LinkFailureNote key={failure.code === 'open-failed' ? failure.url : failure.code} failure={failure} />}
          <button type="button" className="btn primary wide" onClick={() => onOpenLink('pawinhand')}>
            포인핸드에서 보기
          </button>
          {animal.hasSourceNotice && (
            <button type="button" className="small" onClick={() => onOpenLink('source')}>
              국가동물보호정보시스템 원문 공고 보기
            </button>
          )}
        </footer>
      </aside>
    </>
  )
}

/** 2 사진 — 앞 3장. 못 불러온 사진은 빼고, 하나도 남지 않으면 빈 그림 하나. 작은 사진 줄은 2장 이상일 때만 */
function Gallery({ photos }: { photos: string[] }) {
  const [failed, setFailed] = useState<string[]>([])
  const [selected, setSelected] = useState(0)
  const usable = photos.filter((url) => !failed.includes(url))
  const index = Math.min(selected, Math.max(usable.length - 1, 0))
  const fail = (url: string) => setFailed((list) => (list.includes(url) ? list : [...list, url]))
  return (
    <div className="gallery">
      <PhotoBox url={usable[index]} onFail={fail} />
      {usable.length >= 2 && (
        <div className="thumbs">
          {usable.map((url, i) => (
            <PhotoBox
              key={url}
              url={url}
              className={i === index ? 'sel' : undefined}
              label={`사진 ${i + 1} 크게 보기`}
              onSelect={() => setSelected(i)}
              onFail={fail}
            />
          ))}
        </div>
      )}
    </div>
  )
}

/** 값이 비면 줄을 숨기지 않고 흐린 「정보 없음」 */
function Fact({ label, value }: { label: string; value: string | null }) {
  return (
    <>
      <dt>{label}</dt>
      <dd className={value ? undefined : 'empty'}>{value || '정보 없음'}</dd>
    </>
  )
}

/** 6 브라우저 열기 실패, 또는 코어가 거절한 주소 */
function LinkFailureNote({ failure }: { failure: LinkFailure }) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [copied, setCopied] = useState(false)

  if (failure.code === 'not-allowed') {
    return (
      <p className="inline-err" role="alert">
        열 수 없는 주소예요
      </p>
    )
  }

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(failure.url)
      setCopied(true)
    } catch {
      // 클립보드에 담지 못하면 선택된 채로 두어 Ctrl+C로 복사하게 한다
      inputRef.current?.focus()
      inputRef.current?.select()
    }
  }

  return (
    <>
      <p className="inline-err" role="alert">
        기본 브라우저를 열지 못했어요. 아래 주소를 복사해 브라우저 주소창에 붙여 넣으세요.
      </p>
      <div className="copy">
        <input
          ref={inputRef}
          readOnly
          value={failure.url}
          aria-label="열려던 주소"
          onFocus={(e) => e.currentTarget.select()}
          onClick={(e) => e.currentTarget.select()}
        />
        <button type="button" className="btn" onClick={copy}>
          {copied ? '복사했어요' : '주소 복사'}
        </button>
      </div>
    </>
  )
}
