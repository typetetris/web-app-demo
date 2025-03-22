import { Button, Flex, Form, TextField } from "@adobe/react-spectrum";
import { Identity } from "../models/Identity";
import { v4 } from "uuid";
import { useState } from "react";
import Add from "@spectrum-icons/workflow/Add";
import Close from "@spectrum-icons/workflow/Close";

export interface CreateNewIdentityFormProps {
  onSubmit: (identity: Identity) => void;
}

export function CreateNewIdentityForm({
  onSubmit,
}: CreateNewIdentityFormProps) {
  const [nextId, setNextId] = useState(v4());
  const [displayName, setDisplayName] = useState("");
  const clearForm = () => {
    setNextId(v4());
    setDisplayName("");
  };
  return (
    <Form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit({
          id: nextId,
          displayName,
        });
        clearForm();
      }}
      onReset={clearForm}
    >
      <TextField
        name="id"
        label="Technical Id"
        isHidden={true}
        value={nextId}
      />
      <Flex direction="row" alignItems="end" gap="size-100">
        <TextField
          name="displayName"
          label="Display Name"
          value={displayName}
          onChange={setDisplayName}
        />
        <Button
          type="submit"
          variant="accent"
          isDisabled={displayName === ""}
          aria-label="Add display name"
        >
          <Add />
        </Button>
        <Button
          type="reset"
          variant="secondary"
          aria-label="Clear display name input field"
        >
          <Close />
        </Button>
      </Flex>
    </Form>
  );
}
