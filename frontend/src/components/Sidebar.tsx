import { Accordion, Disclosure, DisclosurePanel, DisclosureTitle, Flex, Text } from "@adobe/react-spectrum";
import { IdentitiesManager } from "./IdentitiesManager";

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
                    <DisclosurePanel><Text>Chats</Text></DisclosurePanel>
                </Disclosure>
            </Accordion>
        </>
    )
}