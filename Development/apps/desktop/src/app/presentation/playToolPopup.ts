// Pure presentation geometry. No native window, session, game or IPC authority.
export interface PlayToolAnchor {
  readonly top: number;
  readonly bottom: number;
  readonly right: number;
}
export interface PlayToolViewport {
  readonly width: number;
  readonly height: number;
}

export function playToolPopupStyle(
  anchor: PlayToolAnchor, viewport: PlayToolViewport, preferredWidth: number,
): string {
  const edge = 16;
  const gap = 8;
  const width = Math.max(0, Math.min(preferredWidth, viewport.width - edge * 2));
  const left = Math.min(
    Math.max(edge, anchor.right - width),
    Math.max(edge, viewport.width - edge - width),
  );
  const below = Math.max(0, viewport.height - anchor.bottom - gap - edge);
  const above = Math.max(0, anchor.top - gap - edge);
  const positionAbove = below < 176 && above > below;
  const height = positionAbove ? above : below;
  const vertical = positionAbove
    ? `top:auto;bottom:${Math.round(viewport.height - anchor.top + gap)}px;`
    : `top:${Math.round(anchor.bottom + gap)}px;bottom:auto;`;
  return `left:${Math.round(left)}px;right:auto;width:${Math.round(width)}px;${vertical}max-height:${Math.round(height)}px;`;
}
