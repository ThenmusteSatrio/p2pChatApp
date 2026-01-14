import { useState } from "react";

export default function TopBar({find_peer}: {find_peer: (peerId: string) => void}) {
  const [peerId, setPeerId] = useState("");
  return (
    <div className="h-14 border-b border-neutral-800 px-4 flex items-center">
      <input
        value={peerId}
        onChange={(val) => {
          setPeerId(val.target.value)
        }}
        placeholder="Search peer id (Kademlia)"
        className="w-full bg-neutral-800 rounded-md px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-red-600"
      />
      <button onClick={() => find_peer(peerId)} className="bg-red-600 hover:bg-red-500 px-4 rounded-md ml-2">
        Search
      </button>
    </div>
  );
}