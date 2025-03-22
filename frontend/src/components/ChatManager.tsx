import { useEffect, useState } from "react";
import { v4 } from "uuid";
import { Chat } from "../models/Chat";
import { Flex } from "@adobe/react-spectrum";
import { CreateNewChatForm } from "./CreateNewChatForm";
import { ChatList } from "./ChatList";
import { AlertNotification } from "./AlertNotification";

const chatListLocalStorageKey = 'web-app-demo-chat-list';

export interface ChatManagerProps {
    onChatChange: (newChat: Chat | null) => void
}

export function ChatManager({onChatChange}: ChatManagerProps) {

    // On the other hand, not using useQuery and "just" using `useState` and some
    // callbacks makes us easily forget error handling in the (ok unlikely) case
    // json deserialization/serialization fails.
    //
    // And if we abstract over it, we are already halfway into building our own
    // @tanstack/react-query like thingy.
    const [chats, setChats] = useState<Chat[]>([]);
    const [loadError, setLoadError] = useState<ErrorWithMessage | null>(null);
    const [addError, setAddError] = useState<ErrorWithMessage | null>(null);
    const [delError, setDelError] = useState<ErrorWithMessage | null>(null);

    const setAndSaveChats = (chats: Chat[]) => {
        setChats(chats)
        saveChats(chats)
    }

    // Initial load from localStorage on Mount
    useEffect(() => {
        storeError(
            () => {
                const serializedChats = localStorage.getItem(chatListLocalStorageKey) ?? "[]";
                const chats = JSON.parse(serializedChats) as Chat[]
                setChats(chats)
            },
            "Error loading chats.",
            setLoadError
        )
    }, []);

    const addChat = (name: string) => 
        storeError(
            () => {
                const newChats = [...chats, {name, id: v4()}]
                setAndSaveChats(newChats)
            },
            "Error adding a chat.",
            setAddError
        );
    
    const delChat = (id: string) => 
        storeError(
            () => {
                const newChats = chats.filter((chat) => chat.id != id);
                setAndSaveChats(newChats)
            },
            "Error deleting a chat.",
            setDelError
        );

    return (
        <Flex direction='column' gap="size-200">
            {
                [loadError, addError, delError].filter((e) => e != null).map((e) => (
                    <AlertNotification msg={e.msg} />
                ))
            }
            <CreateNewChatForm onSubmit={addChat} />
            {
                chats.length > 0 ? 
                    <ChatList chats={chats} onDelete={delChat} onChatChange={onChatChange}/> :
                    null
            }
        </Flex>
    )
}

function saveChats(chats: Chat[]) {
    localStorage.setItem(chatListLocalStorageKey, JSON.stringify(chats))
}

interface ErrorWithMessage {
    msg: string,
    error: unknown
}

function storeError(op: () => void, msg: string, storeError: (error: ErrorWithMessage | null) => void) {
    try {
        op();
        storeError(null);
    } catch (error) {
        storeError({ msg, error })
    }
}
