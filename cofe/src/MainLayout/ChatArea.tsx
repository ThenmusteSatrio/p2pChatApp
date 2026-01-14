import { invoke } from "@tauri-apps/api/core";
import React, { useEffect, useState } from "react";

type ChatMessage = {
  id: string;
  from: string;
  to: string;
  timestamp: number;
  content: string;
}

export default function ChatArea({chatState, peerId, currentPeerId, messages, getHistoryMessage}: {chatState: boolean, peerId: string, currentPeerId: string, messages: any, getHistoryMessage: (peerId: string) => void}) {
  const [message, setMessage] = useState("");
  const sendMessage = async(e: React.FormEvent) => {
    e.preventDefault();
    const content: ChatMessage = {
      id: crypto.randomUUID(),
      from: peerId,
      to: currentPeerId,
      timestamp: Date.now(),
      content: message
    }
    await invoke("send_message", {
      peerId: currentPeerId,
      message: content
    })

    setMessage("");
    getHistoryMessage(currentPeerId);
  }

  return (
    <div className="flex-1 flex flex-col p-4">
      <div className="flex-1 overflow-y-auto space-y-3">
        {
          messages.map((message: any, i: number) => (
            <Message key={i} from={message.from === peerId ? "me" : "peer"} text={message.content} />
          ))
        }
      </div>

      <div className="mt-4 flex">
        {
          chatState && (
            <>
              <form action="" onSubmit={sendMessage} className="flex w-full">
                  <input
                  value={message}
                  onChange={(val) => {
                    setMessage(val.target.value)
                  }}
                    placeholder="Type a message"
                    className="flex-1 bg-neutral-800 rounded-l-md px-3 py-2 outline-none"
                  />
                  <button type="submit" className="bg-red-600 hover:bg-red-500 px-4 rounded-r-md">
                    Send
                  </button>
              </form>
            </>
          )
        }
      </div>
    </div>
  );
}

function Message({
  from,
  text,
  key
}: {
  from: "me" | "peer";
  text: string;
  key?: number;
}) {
  const isMe = from === "me";

  return (
    <div key={key} className={`flex ${isMe ? "justify-end" : "justify-start"}`}>
      <div
        className={`max-w-xs px-3 py-2 rounded-md text-sm ${
          isMe
            ? "bg-red-600 text-white"
            : "bg-neutral-800"
        }`}
      >
        {text}
      </div>
    </div>
  );
}