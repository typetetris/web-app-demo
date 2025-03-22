import { Content, Flex } from "@adobe/react-spectrum";
import { Chat } from "../models/Chat";
import { Identity } from "../models/Identity";

export interface ChatWindowProps {
    chat: Chat | null,
    identity: Identity | null
}

export function ChatWindow({chat, identity}: ChatWindowProps) {
    return (
        <Flex direction='column'>
            <Content>Chat: {chat?.name ?? 'no chat selected'}</Content>
            <Content>Identity: {identity?.displayName ?? 'no identity selectod'}</Content>
        </Flex>
    )
}