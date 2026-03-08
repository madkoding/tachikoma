import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ChatInput from './ChatInput'

describe('ChatInput', () => {
  const mockOnSend = vi.fn()

  it('renders textarea and send button', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    expect(textarea).toBeInTheDocument()
    expect(button).toBeInTheDocument()
  })

  it('calls onSend when clicking send button with message', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    fireEvent.change(textarea, { target: { value: 'Hello world' } })
    fireEvent.click(button)
    
    expect(mockOnSend).toHaveBeenCalledWith('Hello world')
  })

  it('calls onSend when pressing Enter without Shift', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    
    fireEvent.change(textarea, { target: { value: 'Test message' } })
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: false })
    
    expect(mockOnSend).toHaveBeenCalledWith('Test message')
  })

  it('does not send when pressing Enter with Shift', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    
    fireEvent.change(textarea, { target: { value: 'Test message' } })
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: true })
    
    expect(mockOnSend).not.toHaveBeenCalled()
  })

  it('does not send empty message', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const button = screen.getByRole('button')
    
    fireEvent.click(button)
    
    expect(mockOnSend).not.toHaveBeenCalled()
  })

  it('does not send whitespace-only message', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    fireEvent.change(textarea, { target: { value: '   ' } })
    fireEvent.click(button)
    
    expect(mockOnSend).not.toHaveBeenCalled()
  })

  it('clears message after sending', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    fireEvent.change(textarea, { target: { value: 'Test' } })
    fireEvent.click(button)
    
    expect(textarea).toHaveValue('')
  })

  it('is disabled when disabled prop is true', () => {
    render(<ChatInput onSend={mockOnSend} disabled />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    expect(textarea).toBeDisabled()
    expect(button).toBeDisabled()
  })

  it('cannot send when disabled', () => {
    render(<ChatInput onSend={mockOnSend} disabled />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    fireEvent.change(textarea, { target: { value: 'Test' } })
    fireEvent.click(button)
    
    expect(mockOnSend).not.toHaveBeenCalled()
  })

  it('button is disabled when message is empty', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const button = screen.getByRole('button')
    expect(button).toBeDisabled()
  })

  it('button is enabled when message has content', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    
    fireEvent.change(textarea, { target: { value: 'Hello' } })
    
    expect(button).not.toBeDisabled()
  })

  it('allows newlines with Shift+Enter', () => {
    render(<ChatInput onSend={mockOnSend} />)
    
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    
    fireEvent.change(textarea, { target: { value: 'Line 1' } })
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: true })
    
    // Message should still have the original value, not been sent
    expect(mockOnSend).not.toHaveBeenCalled()
  })
})
