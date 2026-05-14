"use client"

import {
    ChakraProvider,
    Portal,
    Stack,
    Toast,
    Toaster,
    defaultSystem,
} from "@chakra-ui/react"
import type { PropsWithChildren } from "react"

import { toaster } from "@/utils/notify"

export function Provider({ children }: PropsWithChildren) {
    return (
        <ChakraProvider value={defaultSystem}>
            {children}
            <Portal>
                <Toaster toaster={toaster}>
                    {(toast) => (
                        <Toast.Root width={{ md: "sm" }}>
                            <Toast.Indicator />
                            <Stack gap="1" flex="1" maxWidth="100%">
                                {toast.title !== undefined && (
                                    <Toast.Title>{toast.title}</Toast.Title>
                                )}
                                {toast.description !== undefined && (
                                    <Toast.Description>{toast.description}</Toast.Description>
                                )}
                            </Stack>
                            <Toast.CloseTrigger />
                        </Toast.Root>
                    )}
                </Toaster>
            </Portal>
        </ChakraProvider>
    )
}
