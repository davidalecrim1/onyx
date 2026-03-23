import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  filePath: string;
}

export default function ImageViewer({ filePath }: Props) {
  const [dataUrl, setDataUrl] = useState<string | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    setDataUrl(null);
    setError(false);
    invoke<string>("read_binary_as_data_url", { path: filePath })
      .then(setDataUrl)
      .catch(() => setError(true));
  }, [filePath]);

  return (
    <div className="flex h-full items-center justify-center overflow-auto p-8">
      {error ? (
        <p className="text-sm text-text-secondary">Failed to load image</p>
      ) : dataUrl ? (
        <img
          src={dataUrl}
          alt=""
          className="max-h-full max-w-full object-contain"
        />
      ) : null}
    </div>
  );
}
