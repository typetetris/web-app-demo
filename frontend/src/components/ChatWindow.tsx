import { View } from "@adobe/react-spectrum";
import { StyleProps } from "@react-types/shared";

export interface ChatWindowProps extends StyleProps {}

export function ChatWindow(props: StyleProps) {
    return (
        <View backgroundColor="purple-600" {...props} />
    )
}