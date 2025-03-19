import { Accordion, Disclosure, DisclosurePanel, DisclosureTitle } from "@adobe/react-spectrum";
import { IdentitiesManager } from "./IdentitiesManager";
import { ChatManager } from "./ChatManager";

export function Sidebar() {
    return (
        <>
            <Accordion allowsMultipleExpanded>
                <Disclosure id="Identities">
                    <DisclosureTitle>Identities</DisclosureTitle>
                    <DisclosurePanel>
                        <IdentitiesManager />
                    </DisclosurePanel>
                </Disclosure>
                <Disclosure id="Chats">
                    <DisclosureTitle>Chats</DisclosureTitle>
                    <DisclosurePanel>
                        <ChatManager />
                    </DisclosurePanel>
                </Disclosure>
            </Accordion>
        </>
    )
}