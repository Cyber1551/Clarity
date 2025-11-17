import { createTreeCollection, Flex, ScrollArea, TreeView } from "@chakra-ui/react";
import { LuChevronRight, LuFileImage, LuFileVideo, LuFolder } from "react-icons/lu";
import { cn } from "@/lib/utils.ts";
import { TreeNode } from "@/types/mediaTypes.ts";

const FileTree = () => {

    const treeCollection = createTreeCollection<TreeNode>({
        nodeToValue: (node) => node.path,
        rootNode: {
            dirName: "Library",
            path: "/",
            children: []
        },
    })

    const onTreeItemClicked = (node: any) => {
        console.log(node);
    }

    return (
        <Flex as="aside"
              w="256px"
              p="8px"
              borderRightWidth="1px"
              borderColor="gray.100"
              direction="column">
            <ScrollArea.Root className={"flex-1"}>
                <ScrollArea.Viewport>
                    <div className={cn(
                        "flex items-center gap-[0.5rem] px-2 py-1.5 rounded-md cursor-default text-sm transition-colors",
                        "bg-black hover:bg-[var(--chakra-colors-bg-muted)] hover:text-accent-foreground",
                        // currentPath.length === 0 &&
                        // "bg-accent text-accent-foreground font-medium"
                    )}
                         onClick={() => {
                         }}>
                        <LuFolder />
                        <span>Library</span>
                    </div>
                    <TreeView.Root collection={treeCollection} maxW={"sm"} expandOnClick={false} animateContent>
                        <TreeView.Tree>
                            <TreeView.Node
                                indentGuide={<TreeView.BranchIndentGuide />}
                                render={({ node, nodeState }) => nodeState.isBranch ? (
                                    <TreeView.BranchControl
                                        onDoubleClick={() => nodeState.expanded = !nodeState.expanded}
                                        onClick={() => onTreeItemClicked(node)}>
                                        <TreeView.BranchTrigger>
                                            <TreeView.BranchIndicator asChild>
                                                <LuChevronRight />
                                            </TreeView.BranchIndicator>
                                        </TreeView.BranchTrigger>
                                        <LuFolder />
                                        <TreeView.BranchText>{node.name}</TreeView.BranchText>
                                    </TreeView.BranchControl>
                                ) : (
                                    <TreeView.Item>
                                        {node.type === 'image' ? <LuFileImage /> : <LuFileVideo />}
                                        <TreeView.ItemText>{node.name}</TreeView.ItemText>
                                    </TreeView.Item>
                                )} />
                        </TreeView.Tree>
                    </TreeView.Root>
                </ScrollArea.Viewport>
                <ScrollArea.Scrollbar>
                    <ScrollArea.Thumb />
                </ScrollArea.Scrollbar>
            </ScrollArea.Root>
        </Flex>
    )
}

export default FileTree;