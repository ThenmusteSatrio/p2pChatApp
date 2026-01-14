export default function Sidebar({mockNodes, setChatState, setCurrentPeer}: {mockNodes: [], setChatState: (state: boolean) => void, setCurrentPeer: (peerId: string) => void}) {
  return (
    <div className="w-72 border-r border-neutral-800 p-4">
      <h2 className="text-sm font-semibold text-neutral-400 mb-3">
        Kademlia Buckets
      </h2>

      {mockNodes.map((n, i) => (
        <div
          key={i}
          onClick={() => {
            setChatState(true);
            setCurrentPeer(n);
          }}
          className="p-2 rounded-md hover:bg-neutral-800 cursor-pointer mb-1"
        >
          <div className="text-xs text-neutral-500">
            {/* Bucket {n.bucket} */}
          </div>
          <div className="text-sm truncate">
            {n}
          </div>
        </div>
      ))}
    </div>
  );
}