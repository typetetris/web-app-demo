import { Flex, InlineAlert, Text } from "@adobe/react-spectrum";
import Alert from "@spectrum-icons/workflow/Alert";

export interface AlertNotificationProps {
  msg: string;
}
export function AlertNotification(props: AlertNotificationProps) {
  return (
    <InlineAlert>
      <Flex direction="row" alignItems="end" gap="size-200">
        <Alert color="negative" />
        <Text>{props.msg}</Text>
      </Flex>
    </InlineAlert>
  );
}
