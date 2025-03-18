import { ActionButton, Item, ListView, Text } from "@adobe/react-spectrum";
import { Chat } from "../models/Chat";
import Delete from "@spectrum-icons/workflow/Delete";

export interface ChatListProps {
    chats: Chat[],
    onDelete: (id: string) => void,
}
export function ChatList({chats, onDelete} : ChatListProps) {
    return (
        // Using the ListView items property prevents ActionButton from
        // rerendering with a new `onDelete`.
        //
        // Revisit this later in case of performance problems.
        <ListView
            selectionMode="single"
            aria-label="List of Chats"
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