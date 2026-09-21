import { invoke } from "@tauri-apps/api/core";

export type Conversation = {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
};

export async function ensureActiveConversation(): Promise<string> {
  let conversations = await invoke<Conversation[]>("list_conversations");
  if (conversations.length === 0) {
    const created = await invoke<Conversation>("create_conversation", { title: "Main" });
    conversations = [created];
  }
  const id = conversations[0]!.id;
  await invoke("set_active_conversation", { conversationId: id });
  return id;
}
