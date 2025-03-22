import { ActionButton, Item, ListView, Text, Selection } from "@adobe/react-spectrum";
import { Chat } from "../models/Chat";
import Delete from "@spectrum-icons/workflow/Delete";

export interface ChatListProps {
    chats: Chat[],
    onDelete: (id: string) => void,
    onChatChange: (newChat: Chat | null) => void
}
export function ChatList({chats, onDelete, onChatChange} : ChatListProps) {
    return (
        // Using the ListView items property prevents ActionButton from
        // rerendering with a new `onDelete`.
        //
        // Revisit this later in case of performance problems.
        <ListView
            selectionMode="single"
            aria-label="List of Chats"
            onSelectionChange={(selection: Selection) => {
                if (selection == "all") {
                    console.warn("ChatList selection all")
                    onChatChange(null)
                } else {
                    const firstKey = selection.size > 0 ? [...selection.values()][0] : null
                    if(firstKey == null) {
                        console.info("ChatList selection empty")
                        onChatChange(null)
                    }
                    else if (typeof firstKey === 'number') {
                        console.warn("ChatList selection of type number")
                        onChatChange(null)
                    } else {
                        onChatChange(chats.find((chat) => chat.id == firstKey) ?? null)
                    }
                }
            }}
        >
            {chats.map((item) => (
                <Item textValue={item.name} key={item.id}>
                    <Text>{item.name}</Text>
                    <ActionButton
                        onPress={() => {
                            onDelete(item.id)
                        }}
                    ><Delete /></ActionButton>
                </Item>
            ))}
        </ListView>
    )
}