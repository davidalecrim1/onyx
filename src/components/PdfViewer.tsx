import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as pdfjsLib from "pdfjs-dist";
import type { PDFDocumentProxy } from "pdfjs-dist";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.min.mjs",
  import.meta.url,
).toString();

interface Props {
  filePath: string;
}

/// Renders a local PDF read-only via PDF.js, loaded as a base64 data URL over Tauri IPC.
export default function PdfViewer({ filePath }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    let pdf: PDFDocumentProxy | null = null;

    async function render() {
      setError(null);

      let dataUrl: string;
      try {
        dataUrl = await invoke<string>("read_binary_as_data_url", {
          path: filePath,
        });
      } catch (err) {
        if (!cancelled) setError(`Failed to load PDF: ${err}`);
        return;
      }

      try {
        pdf = await pdfjsLib.getDocument(dataUrl).promise;
      } catch (err) {
        if (!cancelled) setError(`Failed to parse PDF: ${err}`);
        return;
      }

      if (cancelled || !containerRef.current) return;

      containerRef.current.innerHTML = "";

      for (let pageNum = 1; pageNum <= pdf.numPages; pageNum++) {
        if (cancelled) break;

        const page = await pdf.getPage(pageNum);
        const viewport = page.getViewport({ scale: 1.5 });

        const canvas = document.createElement("canvas");
        canvas.width = viewport.width;
        canvas.height = viewport.height;

        const wrapper = document.createElement("div");
        wrapper.className = "flex justify-center mb-4";
        wrapper.style.maxWidth = "100%";
        wrapper.appendChild(canvas);
        containerRef.current.appendChild(wrapper);

        const ctx = canvas.getContext("2d");
        if (!ctx) continue;

        await page.render({ canvasContext: ctx, viewport, canvas }).promise;
      }
    }

    render();
    return () => {
      cancelled = true;
      pdf?.destroy();
    };
  }, [filePath]);

  if (error) {
    return (
      <div className="flex h-full items-center justify-center text-text-secondary">
        <p className="text-sm">{error}</p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-6">
      <div ref={containerRef} />
    </div>
  );
}
