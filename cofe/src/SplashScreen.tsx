import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Props = {
  onDone: () => void;
};

export default function SplashScreen({ onDone }: Props) {
  const [firstRun, setFirstRun] = useState<boolean | null>(null);
  const [password, setPassword] = useState("");
  const [splashDone, setSplashDone] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    invoke<boolean>("get_first_run")
      .then(setFirstRun)
      .catch(() => setFirstRun(true));
  }, []);

  useEffect(() => {
    const t = setTimeout(() => setSplashDone(true), 1800);
    return () => clearTimeout(t);
  }, []);

  async function submitPassword(e: React.FormEvent) {
    e.preventDefault();
    if (!password) return;
    console.log("setup password", password);
    try {
      setLoading(true);
      await invoke("setup_password", { password });
      onDone();
    } finally {
      setLoading(false);
    }
  }

  if (!splashDone || firstRun === null) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-neutral-950">
        <img src="/logo.png" className="w-28 h-28 animate-fade-in" />
      </div>
    );
  }

  if (firstRun) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-neutral-950">
        <form onSubmit={submitPassword} className="flex gap-2">
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Create password"
            className="bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-red-600"
          />
          <button
            type="submit"
            disabled={loading}
            className="bg-red-600 hover:bg-red-500 px-4 rounded-md disabled:opacity-50"
          >
            {loading ? "Saving…" : "Submit"}
          </button>
        </form>
      </div>
    );
  }

  onDone();
  return null;
}
