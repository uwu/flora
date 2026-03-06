import type { TextActionModalState, TreeSelection } from './types'

type TextActionModalProps = {
  state: TextActionModalState
  onChangeValue: (value: string) => void
  onClose: () => void
  onSubmit: () => void
}

export function TextActionModal({ state, onChangeValue, onClose, onSubmit }: TextActionModalProps) {
  return (
    <div className='fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4'>
      <div className='w-full max-w-md rounded-xl border bg-popover p-4 shadow-2xl'>
        <div className='mb-3 text-sm font-semibold'>
          {state.mode === 'create_file'
            ? 'Create File'
            : state.mode === 'create_folder'
            ? 'Create Folder'
            : 'Rename'}
        </div>
        <textarea
          rows={2}
          value={state.value}
          onChange={(event) => onChangeValue(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Escape') {
              event.preventDefault()
              onClose()
            }
            if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
              event.preventDefault()
              onSubmit()
            }
          }}
          className='w-full resize-y rounded-md border bg-background px-2 py-1.5 font-mono text-xs'
        />
        <div className='mt-3 flex justify-end gap-2'>
          <button
            type='button'
            onClick={onClose}
            className='rounded-md border px-2.5 py-1 text-xs hover:bg-accent'
          >
            Cancel
          </button>
          <button
            type='button'
            onClick={onSubmit}
            className='rounded-md border px-2.5 py-1 text-xs font-medium hover:bg-accent'
          >
            Save
          </button>
        </div>
      </div>
    </div>
  )
}

type DeleteConfirmModalProps = {
  target: TreeSelection
  onClose: () => void
  onConfirm: () => void
}

export function DeleteConfirmModal({ target, onClose, onConfirm }: DeleteConfirmModalProps) {
  return (
    <div className='fixed inset-0 z-50 flex items-center justify-center bg-black/35 p-4'>
      <div className='w-full max-w-sm rounded-xl border bg-popover p-4 shadow-2xl'>
        <div className='mb-2 text-sm font-semibold'>Delete {target.kind}?</div>
        <div className='mb-4 rounded border bg-muted/40 p-2 font-mono text-xs'>
          {target.path}
        </div>
        <div className='flex justify-end gap-2'>
          <button
            type='button'
            onClick={onClose}
            className='rounded-md border px-2.5 py-1 text-xs hover:bg-accent'
          >
            Cancel
          </button>
          <button
            type='button'
            onClick={onConfirm}
            className='rounded-md border border-destructive/40 bg-destructive/10 px-2.5 py-1 text-xs font-medium text-destructive hover:bg-destructive/20'
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  )
}
