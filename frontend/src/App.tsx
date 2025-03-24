import {
  Header,
  Heading,
  Grid,
  View,
  Flex,
  Divider,
  ToastContainer,
} from "@adobe/react-spectrum";
import { Sidebar } from "./components/Sidebar";
import { ChatWindow } from "./components/ChatWindow";
import { Identity } from "./models/Identity";
import { useState } from "react";
import { Chat } from "./models/Chat";
import { useChat } from "./hooks/useChat";
import { useNavigate, useParams } from "react-router";

function App() {
  const [identity, setIdentity] = useState<Identity | null>(null);
  const { chatName, chatId } = useParams();
  const chat: Chat | null =
    chatId != null && chatName != null ? { id: chatId, name: chatName } : null;
  const navigate = useNavigate();

  return (
    <View height="100vh" width="100wh" paddingX="16px">
      <Grid
        areas={["header  header", "nav content", "footer footer"]}
        columns={["1fr", "3fr"]}
        rows={["size-1000", "auto", "size-2000"]}
        gap="size-100"
        height="100%"
      >
        <Header gridArea={"header"}>
          <Heading level={1}>Web App Demo</Heading>
          <Divider></Divider>
        </Header>
        <View gridArea={"nav"}>
          <Sidebar
            onChatChange={(chat) => {
              let targetUrl;
              if (chat != null) {
                targetUrl = `/${encodeURIComponent(chat.name)}/${encodeURIComponent(chat.id)}`;
              } else {
                targetUrl = `/`;
              }
              navigate(targetUrl);
            }}
            onIdentityChange={(identity) => setIdentity(identity)}
            chat={chat}
          />
        </View>
        <Flex gridArea={"content"} direction="column" minHeight="100%">
          {chat != null && identity != null ? (
            <ChatClient chat={chat} identity={identity} />
          ) : (
            <IdentityOrChatMissing />
          )}
        </Flex>
        <Flex
          gridArea="footer"
          direction="row"
          justifyContent="center"
          margin="size-300"
        >
          <ToastContainer />
        </Flex>
      </Grid>
    </View>
  );
}

export default App;

interface ChatProps {
  chat: Chat;
  identity: Identity;
}

function ChatClient({ chat, identity }: ChatProps) {
  const chatClient = useChat(chat.id, identity.id, identity.displayName);

  if (chatClient.isError) {
    throw chatClient.error;
  }

  if (chatClient.isPending) {
    return <Pending />;
  }

  if (chatClient.isClosed) {
    return <Closed />;
  }

  return (
    <ChatWindow
      messages={chatClient.messages}
      identity={identity}
      chat={chat}
      height="100%"
      marginX="size-300"
      onSend={chatClient.send}
    />
  );
}

function IdentityOrChatMissing() {
  return <h1>Please select a identity and a chat!</h1>;
}

function Pending() {
  return <h1>Pending</h1>;
}

function Closed() {
  return <h1>Closed</h1>;
}
