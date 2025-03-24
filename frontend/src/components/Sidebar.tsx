import {
  Accordion,
  Disclosure,
  DisclosurePanel,
  DisclosureTitle,
  Heading,
} from "@adobe/react-spectrum";
import { IdentitiesManager, IdentitiesManagerProps } from "./IdentitiesManager";
import { ChatManager, ChatManagerProps } from "./ChatManager";

export interface SidebarProps
  extends IdentitiesManagerProps,
    ChatManagerProps {}

export function Sidebar({
  onIdentityChange,
  onChatChange,
  chat,
}: SidebarProps) {
  return (
    <>
      <Heading level={2}>Einstellungen</Heading>
      <Accordion
        allowsMultipleExpanded
        defaultExpandedKeys={["Identities", "Chats"]}
      >
        <Disclosure id="Identities">
          <DisclosureTitle>Identities</DisclosureTitle>
          <DisclosurePanel>
            <IdentitiesManager onIdentityChange={onIdentityChange} />
          </DisclosurePanel>
        </Disclosure>
        <Disclosure id="Chats">
          <DisclosureTitle>Chats</DisclosureTitle>
          <DisclosurePanel>
            <ChatManager onChatChange={onChatChange} chat={chat} />
          </DisclosurePanel>
        </Disclosure>
      </Accordion>
    </>
  );
}
