import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Props = {
  onDone: () => void;
};

type IpVersion = "ipv4" | "ipv6";
type NetworkConfig = {
  ip_version: "ipv4" | "ipv6";
  listen_ip: string;
  listen_port: number;
  bootstrap_ip?: string | null;
  bootstrap_port?: number | null;
  bootstrap_peer_id?: string | null;
};


export default function SplashScreen({ onDone }: Props) {
  const [firstRun, setFirstRun] = useState<boolean | null>(null);
  const [password, setPassword] = useState("");
  const [splashDone, setSplashDone] = useState(false);
  const [loading, setLoading] = useState(false);


  const [step, setStep] = useState<"password" | "network">("password");
  const [ipVersion, setIpVersion] = useState<IpVersion>("ipv4");
  const [ipAddress, setIpAddress] = useState("0.0.0.0");
  const [port, setPort] = useState("8000");
  const [bootstrapIp, setBootstrapIp] = useState("");
  const [bootstrapPort, setBootstrapPort] = useState("");
  const [bootstrapPeer, setBootstrapPeer] = useState("");

  useEffect(() => {
  invoke<{ network: NetworkConfig }>("load_config")
    .then((cfg) => {
      const net = cfg.network;

      setIpVersion(net.ip_version);
      setIpAddress(net.listen_ip);
      setPort(String(net.listen_port));
       console.log("Listen Port", net.listen_port);
      setBootstrapIp(net.bootstrap_ip ?? "");
      setBootstrapPort(net.bootstrap_port ? String(net.bootstrap_port) : "");
      setBootstrapPeer(net.bootstrap_peer_id ?? "");
    })
    .catch((err) => {
      console.warn("Failed to load config, using defaults", err);
    });
}, []);


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
      setStep("network");
    } finally {
      setLoading(false);
    }
  }

  async function saveNetworkConfig() {
    console.log("saving network config");
    await invoke("save_config", {
      cfg: {
        network: {
          ip_version: ipVersion,
          listen_ip: ipAddress,
          listen_port: Number(port),

          bootstrap_ip: bootstrapIp || null,
          bootstrap_port: bootstrapPort ? Number(bootstrapPort) : null,
          bootstrap_peer_id: bootstrapPeer || null
        }
      }
    });
    console.log("saved network config");

    onDone();
  }


  if (!splashDone || firstRun === null) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-neutral-950">
        <img src="/logo.png" className="w-28 h-28 animate-fade-in" />
      </div>
    );
  }

    if (firstRun && step === "password") {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-neutral-950 text-white">
        <div className="w-[380px] bg-neutral-900 rounded-xl p-6 space-y-6 shadow-xl">
          <div className="flex flex-col items-center gap-3">
            <img src="/logo.png" className="w-14 h-14" />
            <h2 className="text-lg font-semibold">Welcome</h2>
            <p className="text-sm text-neutral-400 text-center">
              Create a password to secure your local data
            </p>
          </div>

          <form onSubmit={submitPassword} className="space-y-4">
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Create password"
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-red-600"
            />

            <button
              type="submit"
              disabled={loading}
              className="w-full bg-red-600 hover:bg-red-500 py-2 rounded-md disabled:opacity-50"
            >
              {loading ? "Saving…" : "Continue"}
            </button>
          </form>
        </div>
      </div>
    );
  }

  if (firstRun && step === "network") {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-neutral-950 text-white">
        <div className="w-[420px] bg-neutral-900 rounded-xl p-6 space-y-6 shadow-xl">
          <div className="flex items-center gap-3">
            <img src="/logo.png" className="w-10 h-10" />
            <h2 className="text-lg font-semibold">Network setup</h2>
          </div>

          {/* IP version */}
          <div className="flex gap-6 text-sm">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                checked={ipVersion === "ipv4"}
                onChange={() => {
                  setIpVersion("ipv4");
                  setIpAddress("0.0.0.0");
                }}
              />
              IPv4
            </label>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                checked={ipVersion === "ipv6"}
                onChange={() => {
                  setIpVersion("ipv6");
                  setIpAddress("::");
                }}
              />
              IPv6
            </label>
          </div>

          {/* Network fields */}
          <div className="space-y-3">
            <input
              value={ipAddress}
              onChange={(e) => setIpAddress(e.target.value)}
              placeholder={ipVersion === "ipv6" ? "::" : "0.0.0.0"}
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none"
            />

            <input
              value={port}
              onChange={(e) => setPort(e.target.value)}
              placeholder="Port"
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none"
            />

            <input
              value={bootstrapIp}
              onChange={(e) => setBootstrapIp(e.target.value)}
              placeholder="Bootstrap IP (optional)"
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none"
            />

            <input
              value={bootstrapPort}
              onChange={(e) => setBootstrapPort(e.target.value)}
              placeholder="Bootstrap port (optional)"
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none"
            />

            <input
              value={bootstrapPeer}
              onChange={(e) => setBootstrapPeer(e.target.value)}
              placeholder="Bootstrap peer id (optional)"
              className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none"
            />
          </div>

          <button
            onClick={saveNetworkConfig}
            className="w-full bg-red-600 hover:bg-red-500 py-2 rounded-md"
          >
            Save & Continue
          </button>
        </div>
      </div>
    );
  }


  onDone();
  return null;
}
