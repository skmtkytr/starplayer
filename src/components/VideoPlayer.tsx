import { useRef, useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { MediaFile } from "../types";

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

  const videoSrc = convertFileSrc(file.path);

  useEffect(() => {
    setError(null);
    if (videoRef.current) {
      videoRef.current.load();
      videoRef.current.play().catch(() => {});
    }
  }, [file.id]);

  // Keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const v = videoRef.current;
      if (e.key === "Escape") {
        onClose();
      } else if (e.key === "n" && onNext) {
        onNext();
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
  }, [onClose, onNext, onPrev]);

  const handleEnded = () => {
    if (onNext) onNext();
  };

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
          onError={() =>
            setError(
              `Cannot play this file. Format may not be supported: .${file.extension}`
            )
          }
        />
      )}
    </div>
  );
}
