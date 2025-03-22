import {
  Item,
  ListView,
  Text,
  ActionButton,
  Selection,
} from "@adobe/react-spectrum";
import { Identity } from "../models/Identity";
import Delete from "@spectrum-icons/workflow/Delete";
import { useCallback, useEffect, useState } from "react";

export interface IdentitiesListProps {
  identities: Identity[];
  onDelete: (id: string) => void;
  onIdentityChange: (newIdentity: Identity | null) => void;
}

function getSelectedIdentity(
  selection: Selection,
  identities: Identity[],
): string | null {
  // The selection is valid, if it is one item and it is present in identities
  // Otherwise if selection isn't empty, we take the first element in selection,
  // that is also present in identities
  // Otherwise we take the first identity
  const idList = identities.map((identity) => identity.id);
  const idSet = new Set(idList);

  if (selection !== "all") {
    const validIds = [...selection].filter(
      (id) => typeof id === "string" && idSet.has(id),
    ) as string[];
    if (validIds.length > 0) {
      return validIds[0] ?? null;
    }
  }

  return idList[0] ?? null;
}

function getEffectiveSelectedIdentity(
  selectedIdentity: string | null,
  identities: Identity[],
): string | null {
  return identities.find((identity) => identity.id === selectedIdentity)
    ? selectedIdentity
    : (identities[0]?.id ?? null);
}

export function IdentitiesList({
  identities,
  onDelete,
  onIdentityChange,
}: IdentitiesListProps) {
  const [selectedIdentity, setSelectedIdentityRaw] = useState<string | null>(
    getEffectiveSelectedIdentity(null, identities),
  );
  const setSelectedIdentity = useCallback(
    (id: string | null) => {
      const selectedIdentity =
        identities.find((identity) => identity.id == id) ?? null;
      setSelectedIdentityRaw(selectedIdentity?.id ?? null);
      onIdentityChange(selectedIdentity);
    },
    [setSelectedIdentityRaw, onIdentityChange, identities],
  );

  // Update the selection on a change of the identities list.
  const effectivelySelectedIdentity = getEffectiveSelectedIdentity(
    selectedIdentity,
    identities,
  );
  useEffect(() => {
    if (effectivelySelectedIdentity !== selectedIdentity) {
      setSelectedIdentity(effectivelySelectedIdentity);
    }
  }, [selectedIdentity, effectivelySelectedIdentity, setSelectedIdentity]);

  const selection = effectivelySelectedIdentity
    ? [effectivelySelectedIdentity]
    : [];
  // Using the items property of ListView prevents the rerendering of the ActionButton
  // when the size of the identities array grow from 1 to 2. Because its cached
  // on key, which doesn't change.
  //
  // Revisit this later in case of performance problems.
  return identities.length > 0 ? (
    <ListView
      selectionMode="single"
      aria-label="List of Identities for chatting"
      disallowEmptySelection
      selectedKeys={new Set(selection)}
      onSelectionChange={(selection) =>
        setSelectedIdentity(getSelectedIdentity(selection, identities))
      }
    >
      {identities.map((item) => (
        <Item textValue={item.displayName} key={item.id}>
          <Text>{item.displayName}</Text>
          <ActionButton
            onPress={() => {
              onDelete(item.id);
            }}
            isDisabled={identities.length <= 1}
          >
            <Delete />
          </ActionButton>
        </Item>
      ))}
    </ListView>
  ) : null;
}
