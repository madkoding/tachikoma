import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { Modal } from './Modal'
import { Brain } from 'lucide-react'

describe('Modal', () => {
  const mockOnClose = vi.fn()
  
  const defaultProps = {
    isOpen: true,
    onClose: mockOnClose,
    title: 'Test Modal',
    children: <div>Modal content</div>,
  }

  it('renders when isOpen is true', () => {
    render(<Modal {...defaultProps} />)
    
    expect(screen.getByText('Test Modal')).toBeInTheDocument()
    expect(screen.getByText('Modal content')).toBeInTheDocument()
  })

  it('does not render when isOpen is false', () => {
    render(<Modal {...defaultProps} isOpen={false} />)
    
    expect(screen.queryByText('Test Modal')).not.toBeInTheDocument()
  })

  it('calls onClose when clicking close button', () => {
    render(<Modal {...defaultProps} />)
    
    const closeButton = screen.getByRole('button')
    fireEvent.click(closeButton)
    
    expect(mockOnClose).toHaveBeenCalledTimes(1)
  })

  it('calls onClose when clicking backdrop', () => {
    render(<Modal {...defaultProps} />)
    
    const backdrop = document.querySelector('.bg-gray-900')?.parentNode?.previousSibling
    if (backdrop) {
      fireEvent.click(backdrop as Element)
    }
    
    expect(mockOnClose).toHaveBeenCalledTimes(1)
  })

  it('renders icon when provided', () => {
    render(
      <Modal 
        {...defaultProps} 
        icon={<Brain className="w-6 h-6" data-testid="modal-icon" />}
      />
    )
    
    expect(screen.getByTestId('modal-icon')).toBeInTheDocument()
  })

  it('does not render close button when showCloseButton is false', () => {
    render(<Modal {...defaultProps} showCloseButton={false} />)
    
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it('applies correct max-width class', () => {
    const { rerender } = render(<Modal {...defaultProps} maxWidth="lg" />)
    
    const modal = document.querySelector('.max-w-lg')
    expect(modal).toBeInTheDocument()
    
    rerender(<Modal {...defaultProps} maxWidth="xl" />)
    
    expect(document.querySelector('.max-w-xl')).toBeInTheDocument()
    expect(document.querySelector('.max-w-lg')).not.toBeInTheDocument()
  })

  it('renders decorative corners', () => {
    render(<Modal {...defaultProps} />)
    
    const corners = document.querySelectorAll('.border-cyan-500')
    expect(corners.length).toBeGreaterThanOrEqual(4)
  })

  it('renders with custom children', () => {
    render(
      <Modal {...defaultProps}>
        <div data-testid="custom-content">Custom content</div>
      </Modal>
    )
    
    expect(screen.getByTestId('custom-content')).toBeInTheDocument()
  })

  it('has proper z-index for overlay', () => {
    render(<Modal {...defaultProps} />)
    
    const overlay = document.querySelector('.fixed.inset-0')
    expect(overlay).toHaveClass('z-[9999]')
  })
})
