import { useState, type ReactNode } from 'react'

interface Props {
  url: string | undefined
  className?: string
  /** 있으면 누를 수 있는 사진(상세의 작은 사진)이 된다 */
  onSelect?: () => void
  label?: string
  /** 못 불러왔을 때 — 상세의 사진 줄이 그 사진을 빼게 */
  onFail?: (url: string) => void
  children?: ReactNode
}

/** 사진 한 장. 없거나 못 불러오면 「사진 없음」 빈 그림(UI-1의 10) */
export default function PhotoBox({ url, className, onSelect, label, onFail, children }: Props) {
  const [failedUrl, setFailedUrl] = useState<string | null>(null)
  const broken = !url || failedUrl === url
  const classes = ['ph', broken && 'none', className].filter(Boolean).join(' ')
  const content = (
    <>
      {!broken && (
        <img
          src={url}
          alt=""
          loading="lazy"
          decoding="async"
          referrerPolicy="no-referrer"
          onError={() => {
            setFailedUrl(url)
            if (url) onFail?.(url)
          }}
        />
      )}
      {children}
    </>
  )
  return onSelect ? (
    <button type="button" className={classes} onClick={onSelect} aria-label={label}>
      {content}
    </button>
  ) : (
    <div className={classes}>{content}</div>
  )
}
