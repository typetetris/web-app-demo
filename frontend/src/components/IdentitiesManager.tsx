import { Flex } from "@adobe/react-spectrum";
import { IdentitiesList } from "./IdentitiesList";
import { CreateNewIdentityForm } from "./CreateNewIdentityForm";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Identity } from "../models/Identity";
import { AlertNotification } from "./AlertNotification";

const identityListLocalStorageKey = 'web-app-demo-identity-list';

export function IdentitiesManager() {
    
    // That is of course bonkers, using useState and some callbacks handling the
    // localstorage additionally would be sufficient.
    // It was just to use a bit more tanstack/query for demonstration.

    const { refetch, data, isError } = useQuery({
        queryKey: ['identities'],
        queryFn: () => {
            const serializedIdentities = localStorage.getItem(identityListLocalStorageKey) ?? "[]"
            const identities = JSON.parse(serializedIdentities)
            return (identities as Identity[])
        }
    })

    // Errors from the mutations are ignored at the moment. This needs to be fixed.
    const addIdentity = useMutation({
        mutationFn: async (newIdentity: Identity) => {
            const serializedIdentities = localStorage.getItem(identityListLocalStorageKey) ?? "[]"
            const identities = (JSON.parse(serializedIdentities) as Identity[])
            const newIdentities = [...identities, newIdentity]
            localStorage.setItem(identityListLocalStorageKey, JSON.stringify(newIdentities))
            refetch()
            return newIdentities
        }
    })

    const delIdentity = useMutation({
        mutationFn: async (id: string) => {
            const serializedIdentities = localStorage.getItem(identityListLocalStorageKey) ?? "[]"
            const identities = (JSON.parse(serializedIdentities) as Identity[])
            const newIdentities = identities.filter((identity) => identity.id !== id)
            localStorage.setItem(identityListLocalStorageKey, JSON.stringify(newIdentities))
            refetch()
            return newIdentities
        }
    })

    return (
        <Flex direction="column" gap="size-200">
            {isError ? (
                <AlertNotification msg="Problem loading identities" />
            ) : null}
            <CreateNewIdentityForm onSubmit={addIdentity.mutate} />
            <IdentitiesList identities={data ?? []} onDelete={(id) => delIdentity.mutate(id)}/>
        </Flex>
    )
}