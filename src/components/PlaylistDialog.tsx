import { useState } from "react";
import type { Playlist, PlaylistFilter, Workspace } from "../types";

const FIELDS: { value: PlaylistFilter["field"]; label: string }[] = [
  { value: "filename", label: "Filename" },
  { value: "extension", label: "Extension" },
  { value: "path", label: "Path" },
  { value: "series_name", label: "Series name" },
  { value: "workspace_id", label: "Workspace" },
  { value: "recent_days", label: "Recent (days)" },
];

const OPERATORS: { value: PlaylistFilter["operator"]; label: string }[] = [
  { value: "contains", label: "contains" },
  { value: "not_contains", label: "not contains" },
  { value: "equals", label: "equals" },
  { value: "starts_with", label: "starts with" },
  { value: "ends_with", label: "ends with" },
];

interface Props {
  workspaces: Workspace[];
  existing?: Playlist;
  onSubmit: (name: string, filters: PlaylistFilter[]) => void;
  onClose: () => void;
}

export function PlaylistDialog({
  workspaces,
  existing,
  onSubmit,
  onClose,
}: Props) {
  const [name, setName] = useState(existing?.name ?? "");
  const [filters, setFilters] = useState<PlaylistFilter[]>(
    existing?.filters.length ? existing.filters : [{ field: "filename", operator: "contains", value: "" }]
  );

  const handleSubmit = () => {
    const validFilters = filters.filter((f) => f.value.trim() !== "");
    if (name.trim() && validFilters.length > 0) {
      onSubmit(name.trim(), validFilters);
    }
  };

  const updateFilter = (index: number, patch: Partial<PlaylistFilter>) => {
    setFilters((prev) =>
      prev.map((f, i) => (i === index ? { ...f, ...patch } : f))
    );
  };

  const addFilter = () => {
    setFilters((prev) => [
      ...prev,
      { field: "filename", operator: "contains", value: "" },
    ]);
  };

  const removeFilter = (index: number) => {
    setFilters((prev) => prev.filter((_, i) => i !== index));
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal"
        style={{ minWidth: 520 }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2>{existing ? "Edit Playlist" : "New Playlist"}</h2>
        <div className="modal-field">
          <label>Name</label>
          <input
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder='e.g. "Anime", "Movies"'
            autoFocus
            autoCapitalize="off"
            autoCorrect="off"
          />
        </div>

        <div style={{ marginBottom: 12 }}>
          <label
            style={{
              fontSize: 12,
              color: "var(--text-secondary)",
              display: "block",
              marginBottom: 8,
            }}
          >
            Filter conditions (all must match)
          </label>
          {filters.map((f, i) => (
            <div
              key={i}
              style={{
                display: "flex",
                gap: 6,
                marginBottom: 6,
                alignItems: "center",
              }}
            >
              <select
                className="filter-select"
                value={f.field}
                onChange={(e) => {
                  const field = e.target.value as PlaylistFilter["field"];
                  const patch: Partial<PlaylistFilter> = { field };
                  if (field === "workspace_id" || field === "recent_days") {
                    patch.operator = "equals";
                    patch.value = "";
                  }
                  updateFilter(i, patch);
                }}
              >
                {FIELDS.map((fld) => (
                  <option key={fld.value} value={fld.value}>
                    {fld.label}
                  </option>
                ))}
              </select>
              {f.field === "workspace_id" ? (
                <span style={{ fontSize: 13, color: "var(--text-secondary)", padding: "0 4px" }}>is</span>
              ) : f.field === "recent_days" ? (
                <span style={{ fontSize: 13, color: "var(--text-secondary)", padding: "0 4px" }}>within last</span>
              ) : (
                <select
                  className="filter-select"
                  value={f.operator}
                  onChange={(e) =>
                    updateFilter(i, {
                      operator: e.target.value as PlaylistFilter["operator"],
                    })
                  }
                >
                  {OPERATORS.map((op) => (
                    <option key={op.value} value={op.value}>
                      {op.label}
                    </option>
                  ))}
                </select>
              )}
              {f.field === "workspace_id" ? (
                <select
                  className="filter-select"
                  style={{ flex: 1 }}
                  value={f.value}
                  onChange={(e) => updateFilter(i, { value: e.target.value })}
                >
                  <option value="">Select workspace...</option>
                  {workspaces.map((ws) => (
                    <option key={ws.id} value={ws.id}>
                      {ws.name}
                    </option>
                  ))}
                </select>
              ) : f.field === "recent_days" ? (
                <>
                  <input
                    type="number"
                    min={1}
                    step={1}
                    style={{
                      width: 90,
                      padding: "8px 12px",
                      background: "var(--bg-primary)",
                      border: "1px solid var(--border)",
                      borderRadius: 6,
                      color: "var(--text-primary)",
                      fontSize: 13,
                      outline: "none",
                    }}
                    value={f.value}
                    onChange={(e) => updateFilter(i, { value: e.target.value })}
                    placeholder="7"
                  />
                  <span style={{ fontSize: 13, color: "var(--text-secondary)", flex: 1 }}>days</span>
                </>
              ) : (
                <input
                  style={{
                    flex: 1,
                    padding: "8px 12px",
                    background: "var(--bg-primary)",
                    border: "1px solid var(--border)",
                    borderRadius: 6,
                    color: "var(--text-primary)",
                    fontSize: 13,
                    outline: "none",
                  }}
                  value={f.value}
                  onChange={(e) => updateFilter(i, { value: e.target.value })}
                  placeholder="Value..."
                  autoCapitalize="off"
                  autoCorrect="off"
                />
              )}
              {filters.length > 1 && (
                <button
                  className="btn btn-icon btn-small"
                  onClick={() => removeFilter(i)}
                  style={{ flexShrink: 0 }}
                >
                  x
                </button>
              )}
            </div>
          ))}
          <button
            className="btn btn-secondary btn-small"
            onClick={addFilter}
            style={{ marginTop: 4 }}
          >
            + Add condition
          </button>
        </div>

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSubmit}
            disabled={
              !name.trim() || filters.every((f) => f.value.trim() === "")
            }
          >
            {existing ? "Save" : "Create"}
          </button>
        </div>
      </div>
    </div>
  );
}
