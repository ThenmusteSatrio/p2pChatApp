import { useEffect, useRef, useState } from "react";
import ChatArea from "./ChatArea";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";


export default function MainLayout() {
  const [peers, setPeers] = useState<[]>([]);
  const [chatState, setChatState] = useState(false);
  const [peerId, setPeerId] = useState("");
  const [currentPeer, setCurrentPeer] = useState("");
  const [messages, setMessages] = useState([]);

  const currentPeerRef = useRef(currentPeer);
  
  async function find_peer(peerId: string) {
    setPeers(await invoke("find_peer", { peerId }));
  }

  useEffect(() => {
    console.log(peers);
  }, [peers]);

  useEffect(() => {
    async function getPeerId() {
      setPeerId(await invoke("get_self_peer_id"));
    }
    getPeerId();
  }, []);

  async function getHistoryMessage(peerId: string) {
    setMessages(await invoke("get_history_message", { peerId }));
  }


  useEffect(() => {
    console.log("chatState", currentPeer);
    if (chatState && currentPeer != "" ) {
      getHistoryMessage(currentPeer);
    }
  }, [chatState, currentPeer])

  useEffect(() => {
    currentPeerRef.current = currentPeer;
  }, [currentPeer]);

  useEffect(() => {
    console.log(messages[0]);
  }, [messages])

  useEffect(() => {
      const unlistenMsg = listen("message-received", (event) => {
        console.log(event);
        getHistoryMessage(currentPeerRef.current);
      });

    return () => {
      unlistenMsg.then((f) => f());
    };
  }, [])



  return (
    <div className="h-screen w-screen flex bg-neutral-900 text-neutral-100">
      <Sidebar setCurrentPeer={setCurrentPeer} mockNodes={peers} setChatState={setChatState}/>
      <div className="flex flex-col flex-1">
        <TopBar find_peer={find_peer}/>
        <ChatArea getHistoryMessage={getHistoryMessage} messages={messages} peerId={peerId} currentPeerId={currentPeerRef.current}  chatState={chatState} />
      </div>
    </div>
  );
}