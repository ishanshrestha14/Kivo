import { useEffect, useState, useMemo } from "react";
import { Eye, EyeOff, Plus, RotateCcw, Save, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button.jsx";
import { Input } from "@/components/ui/input.jsx";
import { useEnv } from "@/hooks/use-env.js";
import { cn } from "@/lib/utils.js";

function createRow(key = "", value = "") {
  return { id: `env-${Math.random().toString(36).slice(2, 8)}`, key, value, secret: false };
}

function rowsFromVars(vars = []) {
  return vars.map((v) => ({ ...createRow(v.key, v.value), secret: false }));
}

function RowInput({ value, onChange, placeholder, secret, className }) {
  return (
    <Input
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      type={secret ? "password" : "text"}
      className={cn(
        "h-9 rounded-none border-0 border-b border-border/20 bg-transparent text-[12.5px] focus-visible:ring-0 focus-visible:border-primary/50",
        className
      )}
    />
  );
}

function EnvTable({ rows, onChange, onDelete, workspaceVarKeys = [] }) {
  function updateRow(id, field, value) {
    onChange(rows.map((r) => (r.id === id ? { ...r, [field]: value } : r)));
  }

  function removeRow(id) {
    const nextRows = rows.filter((r) => r.id !== id);
    onChange(nextRows);
    onDelete?.(nextRows);
  }

  function addRow() {
    onChange([...rows, createRow()]);
  }

  return (
    <div className="flex flex-col min-h-0">
      { }
      <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_80px_44px] border-b border-border/10 bg-accent/20 px-2 text-[10px] uppercase tracking-[0.16em] text-muted-foreground">
        <div className="py-2.5 font-semibold px-2">Variable</div>
        <div className="py-2.5 font-semibold px-2">Initial Value</div>
        <div className="py-2.5 font-semibold text-center">Secret</div>
        <div className="py-2.5" />
      </div>

      { }
      <div className="thin-scrollbar overflow-auto">
        {rows.map((row, idx) => {
          const overridesGlobal = workspaceVarKeys.includes(row.key.trim()) && row.key.trim();
          return (
            <div
              key={row.id}
              className={cn(
                "group grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_80px_44px] items-center border-b border-border/5 px-2 transition-colors hover:bg-accent/10",
                idx % 2 === 0 ? "bg-background/5" : ""
              )}
            >
              <div className="relative h-10 flex items-center">
                <Input
                  value={row.key}
                  onChange={(e) => updateRow(row.id, "key", e.target.value)}
                  placeholder="e.g. API_URL"
                  className="h-full w-full border-0 bg-transparent text-[13px] font-medium shadow-none focus-visible:ring-0 focus-visible:bg-accent/20 rounded-none px-2"
                />
                {overridesGlobal && (
                  <span className="absolute right-2 top-1/2 -translate-y-1/2 text-[9px] uppercase tracking-wider text-amber-500/80 whitespace-nowrap pointer-events-none bg-background px-1 rounded-sm shadow-sm border border-amber-500/20">
                    overrides global
                  </span>
                )}
              </div>
              <div className="relative h-10 flex items-center">
                <Input
                  value={row.value}
                  onChange={(e) => updateRow(row.id, "value", e.target.value)}
                  placeholder="e.g. localhost:8080"
                  type={row.secret ? "password" : "text"}
                  className="h-full w-full border-0 bg-transparent text-[13px] shadow-none focus-visible:ring-0 font-mono text-muted-foreground focus-visible:text-foreground focus-visible:bg-accent/20 rounded-none px-2"
                />
              </div>
              <div className="flex h-10 items-center justify-center">
                <button
                  type="button"
                  title={row.secret ? "Show value" : "Mask value"}
                  onClick={() => updateRow(row.id, "secret", !row.secret)}
                  className={cn(
                    "flex h-7 w-7 items-center justify-center rounded-md transition-colors",
                    row.secret
                      ? "bg-primary/10 text-primary hover:bg-primary/20"
                      : "text-muted-foreground hover:bg-accent hover:text-foreground"
                  )}
                >
                  {row.secret ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
                </button>
              </div>
              <div className="flex h-10 items-center justify-center">
                <button
                  type="button"
                  onClick={() => removeRow(row.id)}
                  className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground hover:bg-red-500/10 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all justify-self-center"
                >
                  <Trash2 className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>
          );
        })}

        { }
        <div className="p-2">
          <button
            type="button"
            onClick={addRow}
            className="flex items-center gap-1.5 rounded-md px-3 py-1.5 text-[12px] font-medium text-muted-foreground hover:bg-primary/10 hover:text-primary transition-colors"
          >
            <span className="text-lg leading-none mb-0.5">+</span> Add Variable
          </button>
        </div>

      </div>
    </div>
  );
}

export function EnvEditor({ workspaceName, collectionName, initialTab = "workspace", onSave: onSaveProp }) {
  const { vars, isLoading, saveVars, refresh } = useEnv(workspaceName, collectionName);

  const [activeTab, setActiveTab] = useState(initialTab);
  const [workspaceDraft, setWorkspaceDraft] = useState([]);
  const [collectionDraft, setCollectionDraft] = useState([]);
  const [isSaving, setIsSaving] = useState(false);
  const [savedFeedback, setSavedFeedback] = useState(false);

  const isDirty = useMemo(() => {
    if (!vars) return false;

    const cleanWorkspace = workspaceDraft.filter(r => r.key.trim() || r.value.trim());
    const origWorkspace = vars.workspace || [];
    if (cleanWorkspace.length !== origWorkspace.length) return true;
    for (let i = 0; i < cleanWorkspace.length; i++) {
      if (cleanWorkspace[i].key !== origWorkspace[i].key || cleanWorkspace[i].value !== origWorkspace[i].value) return true;
    }

    const cleanCollection = collectionDraft.filter(r => r.key.trim() || r.value.trim());
    const origCollection = vars.collection || [];
    if (cleanCollection.length !== origCollection.length) return true;
    for (let i = 0; i < cleanCollection.length; i++) {
      if (cleanCollection[i].key !== origCollection[i].key || cleanCollection[i].value !== origCollection[i].value) return true;
    }

    return false;
  }, [workspaceDraft, collectionDraft, vars]);

  useEffect(() => {
    setWorkspaceDraft(rowsFromVars(vars.workspace));
  }, [vars.workspace]);

  useEffect(() => {
    setCollectionDraft(rowsFromVars(vars.collection));
  }, [vars.collection]);

  useEffect(() => {
    setActiveTab(initialTab);
  }, [initialTab]);

  async function handleSave(overrideDrafts) {
    setIsSaving(true);
    try {
      if (overrideDrafts) {
        if (activeTab === "workspace") {
          await saveVars("workspace", overrideDrafts.workspace.filter((r) => r.key.trim()));
        } else {
          await saveVars("collection", overrideDrafts.collection.filter((r) => r.key.trim()));
        }
      } else {
        await saveVars("workspace", workspaceDraft.filter((r) => r.key.trim()));
        await saveVars("collection", collectionDraft.filter((r) => r.key.trim()));
      }
      onSaveProp?.();
      setSavedFeedback(true);
      setTimeout(() => setSavedFeedback(false), 1800);
    } finally {
      setIsSaving(false);
    }
  }

  function handleReset() {
    setWorkspaceDraft([]);
    setCollectionDraft([]);
  }

  const workspaceKeys = vars.workspace.map((v) => v.key);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-40 text-[12px] text-muted-foreground animate-pulse">
        Loading variables…
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full min-h-0">
      { }
      <div className="flex items-center justify-between border-b border-border/25 px-4 pt-1 bg-background/20">
        <div className="flex items-center gap-1">
          {[
            { id: "workspace", label: "Workspace Globals" },
            { id: "collection", label: "Collection Variables" },
          ].map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              className={cn(
                "px-3 py-2.5 text-[12px] border-b-2 transition-colors -mb-px",
                activeTab === tab.id
                  ? "border-primary text-foreground font-medium"
                  : "border-transparent text-muted-foreground hover:text-foreground"
              )}
            >
              {tab.label}
            </button>
          ))}
        </div>
        <span className="text-[10px] uppercase font-bold tracking-wider text-muted-foreground/60 pr-2">
          {activeTab === "workspace"
            ? `${workspaceDraft.filter((r) => r.key.trim()).length} workspace variables`
            : `${collectionDraft.filter((r) => r.key.trim()).length} collection variables`}
        </span>
      </div>

      { }
      <div className="flex-1 min-h-0 overflow-hidden bg-background/10">
        {activeTab === "workspace" ? (
          <EnvTable
            rows={workspaceDraft}
            onChange={setWorkspaceDraft}
            onDelete={(nextRows) => handleSave({ workspace: nextRows, collection: collectionDraft })}
          />
        ) : (
          <EnvTable
            rows={collectionDraft}
            onChange={setCollectionDraft}
            onDelete={(nextRows) => handleSave({ workspace: workspaceDraft, collection: nextRows })}
            workspaceVarKeys={workspaceKeys}
          />
        )}
      </div>
      { }
      <div className="flex items-center justify-between border-t border-border/25 px-4 py-3 shrink-0">
        <div className="flex items-center gap-3 text-sm">
          {isDirty && (
            <div className="flex items-center gap-1.5 text-[12px] font-medium text-amber-500">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-amber-500"></span>
              </span>
              Unsaved changes
            </div>
          )}
        </div>
        <div className="flex items-center gap-3">
          <Button
            size="sm"
            className="h-8 px-6 text-[12px] gap-2 shadow-md transition-transform active:scale-95"
            onClick={() => handleSave()}
            disabled={isSaving || !isDirty}
          >
            <Save className="h-3.5 w-3.5" />
            {isSaving ? "Saving…" : savedFeedback ? "Saved!" : "Save"}
          </Button>
        </div>
      </div>
    </div>
  );
}

