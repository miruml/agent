import sys
import yaml
import argparse
from datetime import datetime
from pathlib import Path
from jinja2 import Environment, FileSystemLoader, select_autoescape


class ScriptBuilder:
    def __init__(self, templates_dir="templates", output_dir="output"):
        self.templates_dir = Path(templates_dir)
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)

        # Setup Jinja2 environment
        self.env = Environment(
            loader=FileSystemLoader([
                self.templates_dir,
                self.templates_dir / "base",
                self.templates_dir / "partials",
                self.templates_dir / "scripts"
            ]),
            autoescape=select_autoescape(['html', 'xml']),
            trim_blocks=True,
            lstrip_blocks=True
        )

        # Add custom filters
        self.env.filters['shell_escape'] = self.shell_escape

    def shell_escape(self, value):
        """Escape shell special characters"""
        if not isinstance(value, str):
            value = str(value)
        return value.replace("'", "'\"'\"'")

    def load_config(self, config_path: Path):
        """Load build configuration"""
        with open(config_path, 'r') as f:
            return yaml.safe_load(f)

    def build_script(self, script_config: dict, global_vars: dict):
        """Build a single script from template"""
        script_name = script_config["name"]
        template_name = script_config["template"]
        description = script_config.get("description", "")

        print(f"Building {script_name} from {template_name}...")

        try:
            # Load template
            template = self.env.get_template(template_name)

            # Merge variables
            variables = {**global_vars, **script_config.get("variables", {})}
            variables.update({
                "script_name": script_name,
                "template_name": template_name,
                "description": description
            })

            # Render template
            output = template.render(**variables)

            # Write output file
            output_path = self.output_dir / script_name
            with open(output_path, 'w') as f:
                f.write(output)

            # Make executable
            output_path.chmod(0o755)

            print(f"✅ Generated: {output_path}")
            return True

        except Exception as e:
            print(f"❌ Error building {script_name}: {e}")
            return False

    def build_all(self, config_path=None):
        """Build all scripts from configuration"""
        config = self.load_config(config_path)
        global_vars = config.get("global_variables", {})
        global_vars["build_time"] = datetime.now().isoformat()

        print("Building Miru shell scripts with Jinja2...")
        print(f"Templates dir: {self.templates_dir}")
        print(f"Output dir: {self.output_dir}")
        print()

        success_count = 0
        total_count = len(config["scripts"])

        for script_config in config["scripts"]:
            if self.build_script(script_config, global_vars):
                success_count += 1

        print()
        print(
            f"Build complete: {success_count}/{total_count} "
            "scripts built successfully"
        )

        if success_count > 0:
            print("\nGenerated scripts:")
            for script_config in config["scripts"]:
                output_path = self.output_dir / script_config["name"]
                if output_path.exists():
                    print(f"  {output_path}")

        return success_count == total_count


def main():
    parser = argparse.ArgumentParser(
        description="Build Miru shell scripts from Jinja2 templates",
    )
    parser.add_argument(
        "--config",
        help="Configuration file (YAML or JSON)",
        type=Path,
        required=True,
    )
    parser.add_argument(
        "--output-dir",
        help="Output directory for generated scripts",
        type=Path,
        default="output",
    )
    parser.add_argument(
        "--templates-dir",
        help="Templates directory",
        type=Path,
        default="templates",
    )

    args = parser.parse_args()

    builder = ScriptBuilder(args.templates_dir, args.output_dir)
    success = builder.build_all(args.config)

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
