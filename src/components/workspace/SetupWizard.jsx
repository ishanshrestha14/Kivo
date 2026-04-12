import { useState, useEffect } from "react";
import { FolderOpen, Settings, CheckCircle2, ArrowRight } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button.jsx";
import { Card } from "@/components/ui/card.jsx";
import { Input } from "@/components/ui/input.jsx";

function appendKivoFolder(base) {
  if (!base) return base;
  const isWindows = /^[A-Za-z]:[/\\]/.test(base);
  const sep = isWindows ? "\\" : "/";
  const trimmed = base.replace(/[/\\]+$/, "");
  return `${trimmed}${sep}Kivo`;
}

export function SetupWizard({ onComplete }) {
  const [basePath, setBasePath] = useState("");
  const [appendKivo, setAppendKivo] = useState(true);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const resolvedPath = appendKivo ? appendKivoFolder(basePath) : basePath;

  useEffect(() => {
    if (!basePath) return;
    const lastSegment = basePath.replace(/[/\\]+$/, "").split(/[/\\]/).at(-1) ?? "";
    if (lastSegment.toLowerCase() === "kivo") {
      setAppendKivo(false);
    } else {
      setAppendKivo(true);
    }
  }, [basePath]);

  useEffect(() => {
    async function initDefaultPath() {
      try {
        const defaultPath = await invoke("get_default_storage_path");
        const isWindows = /^[A-Za-z]:[/\\]/.test(defaultPath);
        const sep = isWindows ? "\\" : "/";
        const kivoSuffix = `${sep}Kivo`;
        if (defaultPath.endsWith(kivoSuffix)) {
          setBasePath(defaultPath.slice(0, -kivoSuffix.length));
        } else {
          setBasePath(defaultPath);
          setAppendKivo(false);
        }
      } catch (error) {
        console.error("Failed to get default path:", error);
      }
    }
    initDefaultPath();
  }, []);

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: basePath,
      });
      if (selected) {
        setBasePath(selected);
      }
    } catch (error) {
      console.error("Failed to pick directory:", error);
    }
  };

  const handleFinish = async () => {
    if (!resolvedPath) return;
    setIsSubmitting(true);
    try {
      await invoke("set_storage_path", { path: resolvedPath });
      onComplete();
    } catch (error) {
      console.error("Failed to set storage path:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
      <Card className="w-full max-w-lg border-border/40 bg-card/95 p-8 shadow-2xl">
        <div className="flex flex-col items-center text-center">
          <div className="mb-6 flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10 text-primary">
            <Settings className="h-8 w-8 animate-spin-slow" />
          </div>

          <h1 className="mb-2 text-2xl font-bold tracking-tight text-foreground lg:text-3xl">
            Welcome to Kivo
          </h1>
          <p className="mb-8 text-[14px] text-muted-foreground lg:text-[15px]">
            Let's set up your workspace. Choose where you want to store your API collections and data.
          </p>

          <div className="w-full space-y-4 text-left">
            <div className="space-y-3">
              <label className="text-[12px] font-semibold uppercase tracking-[0.1em] text-muted-foreground">
                Storage Location
              </label>

              <div className="flex gap-2">
                <Input
                  value={basePath}
                  onChange={(e) => setBasePath(e.target.value)}
                  placeholder={basePath || "Loading default path…"}
                  className="h-11 border-border/40 bg-card/50 text-[14px]"
                />
                <Button
                  variant="outline"
                  size="icon"
                  className="h-11 w-11 shrink-0 border-border/40 bg-card/50"
                  onClick={handleBrowse}
                >
                  <FolderOpen className="h-5 w-5" />
                </Button>
              </div>

              <label className="flex cursor-pointer items-center gap-2.5 select-none">
                <div className="relative flex items-center">
                  <input
                    id="append-kivo"
                    type="checkbox"
                    checked={appendKivo}
                    onChange={(e) => setAppendKivo(e.target.checked)}
                    className="peer h-4 w-4 cursor-pointer appearance-none rounded-sm border border-border/60 bg-card/50 checked:border-primary checked:bg-primary transition-colors"
                  />
                  <svg
                    className="pointer-events-none absolute left-0.5 top-0.5 h-3 w-3 text-primary-foreground opacity-0 peer-checked:opacity-100 transition-opacity"
                    viewBox="0 0 12 12"
                    fill="none"
                  >
                    <path d="M2 6l3 3 5-5" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
                  </svg>
                </div>
                <span className="text-[12.5px] text-muted-foreground">
                  Create a <code className="rounded bg-primary/10 px-1 py-0.5 text-[11px] text-primary font-mono">/Kivo</code> subfolder inside the selected location
                </span>
              </label>

              <div className="rounded-md border border-border/20 bg-accent/20 px-3 py-2.5">
                <p className="text-[10px] font-semibold uppercase tracking-[0.12em] text-muted-foreground/60 mb-1">
                  Resolved path
                </p>
                <p className="font-mono text-[12px] text-foreground/80 break-all leading-snug">
                  {resolvedPath || "—"}
                </p>
              </div>

              <p className="text-[11px] text-muted-foreground">
                Your workspaces, collections, and requests will be stored here.
              </p>
            </div>
          </div>

          <div className="mt-8 w-full space-y-3">
            <Button
              className="h-12 w-full gap-2 text-[15px] font-semibold shadow-lg shadow-primary/20"
              onClick={handleFinish}
              disabled={isSubmitting || !resolvedPath}
            >
              {isSubmitting ? "Setting up..." : "Complete Setup"}
              <CheckCircle2 className="h-5 w-5" />
            </Button>

            <div className="flex items-center justify-center gap-2 text-[12px] text-muted-foreground">
              <span>You can change this later in settings</span>
              <ArrowRight className="h-3 w-3" />
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}
