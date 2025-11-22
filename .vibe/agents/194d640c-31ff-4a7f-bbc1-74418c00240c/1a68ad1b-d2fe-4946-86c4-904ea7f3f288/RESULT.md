```python
from dataclasses import dataclass, field
from typing import Optional


@dataclass
class TodoItem:
    """
    Represents a to-do item with a title, description, and completed status.
    """
    title: str
    description: Optional[str] = None
    completed: bool = False


if __name__ == '__main__':
    # Example Usage
    item1 = TodoItem(title="Grocery Shopping", description="Buy milk, eggs, bread", completed=False)
    item2 = TodoItem(title="Pay Bills", completed=True)

    print(item1)
    print(item2)

    item1.completed = True
    print(item1)

```