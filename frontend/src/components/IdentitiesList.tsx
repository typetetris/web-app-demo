import { Item, ListView, Text, ActionButton, Selection, Key } from "@adobe/react-spectrum";
import { Identity } from "../models/Identity";
import Delete from "@spectrum-icons/workflow/Delete";
import { useState } from "react";

export interface IdentitiesListProps {
    identities: Identity[],
    onDelete: (id: string) => void
}

function getSelectedIdentity(selection: Selection, identities: Identity[]): string | null {
    // The selection is valid, if it is one item and it is present in identities
    // Otherwise if selection isn't empty, we take the first element in selection,
    // that is also present in identities
    // Otherwise we take the first identity 
    const idList = identities.map((identity) => identity.id);
    const idSet = new Set(idList);

    if (selection !== 'all') {
        const validIds = [...selection].filter((id) => typeof id === 'string' && idSet.has(id)) as string[]
        if (validIds.length > 0) {
            return (validIds[0] ?? null)
        }
    }

    return (idList[0] ?? null)
}

function getEffectiveSelectedIdentity(selectedIdentity: string | null, identities: Identity[]): string | null {
    return identities.find((identity) => identity.id === selectedIdentity) ?
        selectedIdentity :
        identities[0]?.id ?? null;
}

export function IdentitiesList({ identities, onDelete }: IdentitiesListProps) {
    const [selectedIdentity, setSelectedIdentity] = useState<string | null>(
        getEffectiveSelectedIdentity(null, identities)
    )

    const effectivelySelectedIdentity = getEffectiveSelectedIdentity(selectedIdentity, identities);

    if (effectivelySelectedIdentity !== selectedIdentity) {
        setSelectedIdentity(effectivelySelectedIdentity)
    }

    const selection =
        effectivelySelectedIdentity ?
            [effectivelySelectedIdentity] :
            [];

    return identities.length > 0 ? (
        <ListView
            items={identities}
            selectionMode="single"
            aria-label="List of Identities for chatting"
            disallowEmptySelection
            selectedKeys={new Set(selection)}
            onSelectionChange={(selection) => setSelectedIdentity(getSelectedIdentity(selection, identities))}
        >
            {(item) => (
                <Item textValue={item.displayName}>
                    <Text>{item.displayName}</Text>
                    <ActionButton
                        onPress={() => {
                            onDelete(item.id)
                        }}
                    ><Delete /></ActionButton>
                </Item>
            )
            }
        </ListView>
    ) : null
}