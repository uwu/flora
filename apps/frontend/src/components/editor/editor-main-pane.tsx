type EditorMainPaneProps = {
  theme: string
  deploymentState: {
    isLoading: boolean
    isError: boolean
  }
}

export function EditorMainPane({ theme, deploymentState }: EditorMainPaneProps) {
  return (
    <div className='h-full min-w-0 flex-1'>
      <div className='h-full'>
        <div className='h-full'>
          {deploymentState.isLoading
            ? (
              <div className='flex h-full items-center justify-center text-sm text-muted-foreground'>
                Loading deployment...
              </div>
            )
            : deploymentState.isError
            ? (
              <div className='flex h-full items-center justify-center text-sm text-destructive'>
                Failed to load deployment.
              </div>
            )
            : (
              <monaco-editor
                key={theme}
                className='block h-full w-full'
                theme='vitesse-dark'
                fontSize='14'
                minimap='{"enabled":false}'
                automaticLayout='true'
                padding='12 12'
              />
            )}
        </div>
      </div>
    </div>
  )
}
