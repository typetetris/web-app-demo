import {
  Accordion,
  Disclosure,
  DisclosurePanel,
  DisclosureTitle,
} from "@adobe/react-spectrum";
import { IdentitiesManager, IdentitiesManagerProps } from "./IdentitiesManager";
import { ChatManager, ChatManagerProps } from "./ChatManager";

export interface SidebarProps
  extends IdentitiesManagerProps,
    ChatManagerProps {}

export function Sidebar({ onIdentityChange, onChatChange }: SidebarProps) {
  return (
    <>
      <Accordion allowsMultipleExpanded>
        <Disclosure id="Identities">
          <DisclosureTitle>Identities</DisclosureTitle>
          <DisclosurePanel>
            <IdentitiesManager onIdentityChange={onIdentityChange} />
          </DisclosurePanel>
        </Disclosure>
        <Disclosure id="Chats">
          <DisclosureTitle>Chats</DisclosureTitle>
          <DisclosurePanel>
            <ChatManager onChatChange={onChatChange} />
          </DisclosurePanel>
        </Disclosure>
      </Accordion>
    </>
  );
}
