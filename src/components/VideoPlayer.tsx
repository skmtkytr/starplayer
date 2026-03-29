import { useRef, useEffect, useState, useMemo, useCallback } from "react";
import type { MediaFile } from "../types";

/**
 * Convert a local file path to an asset protocol URL.
 * Encodes each path segment individually to handle special characters
 * like @, #, ?, etc. that break the default convertFileSrc.
 */
function fileToAssetUrl(filePath: string): string {
  // Normalize backslashes to forward slashes
  const normalized = filePath.replace(/\\/g, "/");
  // Encode each path segment individually, preserving / separators
  const segments = normalized.split("/");
  const encoded = segments
    .map((segment) => (segment === "" ? "" : encodeURIComponent(segment)))
    .join("/");
  // Ensure path starts with / (Windows drive paths like C:/ don't have leading /)
  const path = encoded.startsWith("/") ? encoded : `/${encoded}`;
  // Tauri v2 custom protocols:
  //   Windows WebView2: https://<scheme>.localhost/<path>
  //   macOS/Linux:      <scheme>://localhost/<path>
  // Use window.__TAURI_INTERNALS__ to detect Tauri on Windows reliably
  const w = window as unknown as Record<string, unknown>;
  const isTauriWindows =
    typeof w.__TAURI_INTERNALS__ !== "undefined" &&
    navigator.userAgent.includes("Windows");
  if (isTauriWindows) {
    return `https://asset.localhost${path}`;
  }
  return `asset://localhost${path}`;
}

interface Props {
  file: MediaFile;
  onClose: () => void;
  onNext?: () => void;
  onPrev?: () => void;
  queuePosition: number;
  queueTotal: number;
}

export function VideoPlayer({
  file,
  onClose,
  onNext,
  onPrev,
  queuePosition,
  queueTotal,
}: Props) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const [error, setError] = useState<string | null>(null);

  // Stable URL - only recompute when file path changes
  const videoSrc = useMemo(() => fileToAssetUrl(file.path), [file.path]);

  // Only reload when file actually changes
  const prevFileId = useRef(file.id);
  useEffect(() => {
    if (prevFileId.current !== file.id) {
      prevFileId.current = file.id;
      setError(null);
      if (videoRef.current) {
        videoRef.current.load();
        videoRef.current.play().catch(() => {});
      }
    }
  }, [file.id]);

  // Stable callback refs to avoid re-binding keyboard handler
  const onCloseRef = useRef(onClose);
  const onNextRef = useRef(onNext);
  const onPrevRef = useRef(onPrev);
  useEffect(() => {
    onCloseRef.current = onClose;
    onNextRef.current = onNext;
    onPrevRef.current = onPrev;
  });

  // Keyboard shortcuts - single stable listener
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const v = videoRef.current;
      if (e.key === "Escape") {
        onCloseRef.current();
      } else if (e.key === "n" && onNextRef.current) {
        onNextRef.current();
      } else if (e.key === "ArrowRight" && (e.ctrlKey || e.metaKey) && v) {
        e.preventDefault();
        v.currentTime = Math.min(v.currentTime + 60, v.duration || Infinity);
      } else if (e.key === "ArrowLeft" && (e.ctrlKey || e.metaKey) && v) {
        e.preventDefault();
        v.currentTime = Math.max(v.currentTime - 60, 0);
      } else if (e.key === "ArrowRight" && v) {
        e.preventDefault();
        v.currentTime = Math.min(v.currentTime + 5, v.duration || Infinity);
      } else if (e.key === "ArrowLeft" && v) {
        e.preventDefault();
        v.currentTime = Math.max(v.currentTime - 5, 0);
      } else if (e.key === " ") {
        // Skip if video element has focus (native controls handle it)
        if (document.activeElement === videoRef.current) return;
        e.preventDefault();
        if (v) {
          if (v.paused) v.play();
          else v.pause();
        }
      } else if (e.key === "f") {
        videoRef.current?.requestFullscreen();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []); // Empty deps - uses refs for callbacks

  const handleEnded = useCallback(() => {
    onNextRef.current?.();
  }, []);

  const handleError = useCallback(() => {
    setError(
      `Cannot play: .${file.extension} | URL: ${fileToAssetUrl(file.path)}`
    );
  }, [file.extension, file.path]);

  return (
    <div className="video-player">
      <div className="video-player-header">
        <span className="video-player-title">{file.filename}</span>
        {queueTotal > 1 && (
          <span className="video-player-queue">
            {queuePosition} / {queueTotal}
          </span>
        )}
        <div className="video-player-controls">
          {onPrev && (
            <button
              className="btn btn-icon btn-small"
              onClick={onPrev}
              title="Previous (Alt+Left)"
            >
              Prev
            </button>
          )}
          {onNext && (
            <button
              className="btn btn-icon btn-small"
              onClick={onNext}
              title="Next (Alt+Right)"
            >
              Next
            </button>
          )}
          <button
            className="btn btn-icon btn-small"
            onClick={onClose}
            title="Close (Esc)"
          >
            Close
          </button>
        </div>
      </div>
      {error ? (
        <div className="video-player-error">
          <p>{error}</p>
          <p style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
            The format may not be supported by the built-in player.
            <br />
            Supported: MP4 (H.264/H.265), WebM, MOV, OGG
          </p>
        </div>
      ) : (
        <video
          ref={videoRef}
          className="video-element"
          src={videoSrc}
          controls
          autoPlay
          onEnded={handleEnded}
          onError={handleError}
        />
      )}
    </div>
  );
}
