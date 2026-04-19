interface Props {
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm: () => void;
  onClose: () => void;
}

export function ConfirmDialog({
  message,
  confirmLabel = "OK",
  cancelLabel = "Cancel",
  onConfirm,
  onClose,
}: Props) {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <p style={{ whiteSpace: "pre-wrap" }}>{message}</p>
        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            {cancelLabel}
          </button>
          <button
            className="btn btn-primary"
            onClick={() => {
              onConfirm();
              onClose();
            }}
            autoFocus
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
