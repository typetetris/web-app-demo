import { Header, Heading, Grid, View } from "@adobe/react-spectrum";
import { Sidebar } from "./components/Sidebar";
import { ChatWindow } from "./components/ChatWindow";
import { Identity } from "./models/Identity";
import { useState } from "react";
import { Chat } from "./models/Chat";

function App() {
  const [identity, setIdentity] = useState<Identity | null>(null);
  const [chat, setChat] = useState<Chat | null>(null);

  return (
    <Grid
      areas={["header  header", "nav content"]}
      columns={["1fr", "3fr"]}
      rows={["size-1000", "auto", "size-1000"]}
      width="100wh"
      height="100vh"
      gap="size-100"
    >
      <Header gridArea={"header"} margin="size-100">
        <Heading level={1}>Web App Demo</Heading>
      </Header>
      <View gridArea={"nav"}>
        <Sidebar
          onChatChange={(chat) => setChat(chat)}
          onIdentityChange={(identity) => setIdentity(identity)}
        />
      </View>
      <View gridArea={"content"}>
        <ChatWindow chat={chat} identity={identity} />
      </View>
    </Grid>
  );
}

export default App;
