import { Stack } from "expo-router";

export default function RootLayout() {
  return (
    <Stack>
      <Stack.Screen
        name="index"
        options={{ title: "Sessions", headerLargeTitle: true }}
      />
      <Stack.Screen
        name="[sessionId]"
        options={{ title: "Session", headerBackTitle: "Back" }}
      />
    </Stack>
  );
}
