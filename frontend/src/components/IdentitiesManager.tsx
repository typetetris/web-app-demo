import { Text, Flex, InlineAlert } from "@adobe/react-spectrum";
import { IdentitiesList } from "./IdentitiesList";
import { CreateNewIdentityForm } from "./CreateNewIdentityForm";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Identity } from "../models/Identity";
import Alert from "@spectrum-icons/workflow/Alert";

export function IdentitiesManager() {
    
    // That is of course bonkers, using useState and some callbacks handling the
    // localstorage additionally would be sufficient.
    // It was just to use a bit more tanstack/query for demonstration.

    const { refetch, data, isError } = useQuery({
        queryKey: ['identities'],
        queryFn: () => {
            const serializedIdentities = localStorage.getItem('web-app-demo-identities') ?? "[]"
            const identities = JSON.parse(serializedIdentities)
            return (identities as Identity[])
        }
    })

    const addIdentity = useMutation({
        mutationFn: async (newIdentity: Identity) => {
            const serializedIdentities = localStorage.getItem('web-app-demo-identities') ?? "[]"
            const identities = (JSON.parse(serializedIdentities) as Identity[])
            const newIdentities = [...identities, newIdentity]
            localStorage.setItem('web-app-demo-identities', JSON.stringify(newIdentities))
            refetch()
            return newIdentities
        }
    })

    const delIdentity = useMutation({
        mutationFn: async (id: string) => {
            const serializedIdentities = localStorage.getItem('web-app-demo-identities') ?? "[]"
            const identities = (JSON.parse(serializedIdentities) as Identity[])
            const newIdentities = identities.filter((identity) => identity.id !== id)
            localStorage.setItem('web-app-demo-identities', JSON.stringify(newIdentities))
            refetch()
            return newIdentities
        }
    })

    return (
        <Flex direction="column" gap="size-200">
            {isError ? (
                <InlineAlert>
                    <Flex direction='row'>
                        <Text>Problem loading identities</Text>
                        <Alert color='negative' justifySelf='flex-end' />
                    </Flex>
                </InlineAlert>
            ) : null}
            <CreateNewIdentityForm onSubmit={(newIdentity) => {
                addIdentity.mutate(newIdentity)
            }} />
            <IdentitiesList identities={data ?? []} onDelete={(id) => delIdentity.mutate(id)}/>
        </Flex>
    )
}