import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ChatInput from './ChatInput'

describe('ChatInput', () => {
  const mockOnSend = vi.fn()

  beforeEach(() => {
    mockOnSend.mockClear()
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

  it('does not send when pressing Shift+Enter', () => {
    render(<ChatInput onSend={mockOnSend} />)
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    fireEvent.change(textarea, { target: { value: 'Test message' } })
    fireEvent.keyDown(textarea, { key: 'Enter', shiftKey: true })
    expect(mockOnSend).not.toHaveBeenCalled()
  })

  it('is disabled when disabled prop is true', () => {
    render(<ChatInput onSend={mockOnSend} disabled />)
    const textarea = screen.getByPlaceholderText('chat.placeholder')
    const button = screen.getByRole('button')
    expect(textarea).toBeDisabled()
    expect(button).toBeDisabled()
  })
})