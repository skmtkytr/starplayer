import { useState } from "react";

interface Props {
  onSubmit: (name: string) => void;
  onClose: () => void;
}

export function PlaylistDialog({ onSubmit, onClose }: Props) {
  const [name, setName] = useState("");

  const handleSubmit = () => {
    if (name.trim()) {
      onSubmit(name.trim());
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>New Playlist</h2>
        <div className="modal-field">
          <label>Name</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Playlist"
            autoFocus
            onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
          />
        </div>
        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={!name.trim()}
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
