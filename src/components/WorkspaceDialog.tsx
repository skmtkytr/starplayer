import { useState } from "react";

interface Props {
  onSubmit: (name: string, path: string) => void;
  onClose: () => void;
}

export function WorkspaceDialog({ onSubmit, onClose }: Props) {
  const [name, setName] = useState("");
  const [path, setPath] = useState("");

  const handleSubmit = () => {
    if (name.trim() && path.trim()) {
      onSubmit(name.trim(), path.trim());
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h2>Add Workspace</h2>
        <div className="modal-field">
          <label>Name</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="My Videos"
            autoFocus
          />
        </div>
        <div className="modal-field">
          <label>Path</label>
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="/path/to/videos or \\server\share"
          />
        </div>
        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={!name.trim() || !path.trim()}
          >
            Add
          </button>
        </div>
      </div>
    </div>
  );
}
