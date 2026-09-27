import click
from rich.console import Console
from rich.panel import Panel
from pathlib import Path
from .install import find_git_root, install_hooks, uninstall_hooks, get_hook_status

console = Console()

@click.group()
def main():
    """Manage git-notes hooks in your repository."""
    pass

@main.command()
def install():
    """Install git-notes hooks into the current repository."""
    try:
        repo_path = find_git_root(Path.cwd())
        results = install_hooks(repo_path)
        
        success_count = sum(1 for res in results.values() if res)
        total = len(results)
        
        console.print(f"[bold green]✓ Successfully installed {success_count}/{total} hooks in {repo_path}[/bold green]")
        for hook, success in results.items():
            if success:
                console.print(f"  [green]✓[/green] {hook}")
            else:
                console.print(f"  [red]✗[/red] {hook} (failed)")
    except Exception as e:
        console.print(f"[bold red]Error:[/] {e}")

@main.command()
def uninstall():
    """Remove git-notes hooks from the current repository."""
    try:
        repo_path = find_git_root(Path.cwd())
        results = uninstall_hooks(repo_path)
        
        success_count = sum(1 for res in results.values() if res)
        total = len(results)
        
        console.print(f"[bold green]✓ Successfully uninstalled {success_count}/{total} hooks in {repo_path}[/bold green]")
        for hook, success in results.items():
            if success:
                console.print(f"  [green]✓[/green] {hook}")
            else:
                console.print(f"  [yellow]?[/yellow] {hook} (not found or failed)")
    except Exception as e:
        console.print(f"[bold red]Error:[/] {e}")

@main.command()
def status():
    """Show the status of git-notes hooks in the current repository."""
    try:
        repo_path = find_git_root(Path.cwd())
        results = get_hook_status(repo_path)
        
        status_text = ""
        for hook, status_obj in results.items():
            if status_obj["installed"]:
                status_text += f"[green]✓[/green] {hook}: Installed\n"
            elif status_obj["exists"]:
                status_text += f"[yellow]⚠[/yellow] {hook}: Exists but not ours\n"
            else:
                status_text += f"[red]✗[/red] {hook}: Not installed\n"
        
        console.print(Panel.fit(status_text.strip(), title=f"Hook Status for {repo_path}"))
    except Exception as e:
        console.print(f"[bold red]Error:[/] {e}")

if __name__ == '__main__':
    main()
