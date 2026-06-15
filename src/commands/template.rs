use anyhow::{Result, bail};

use crate::cli::{AppNameArgs, TemplateCommand};
use crate::template as templates;

pub fn run(command: TemplateCommand) -> Result<()> {
    match command {
        TemplateCommand::List => {
            for template in templates::list_templates() {
                println!("{}\t{}", template.name, template.description);
            }
            Ok(())
        }
        TemplateCommand::Show(AppNameArgs { name }) => {
            let Some(template) = templates::get_template(&name) else {
                bail!("unknown template {name}");
            };
            println!("name: {}", template.name);
            println!("description: {}", template.description);
            Ok(())
        }
    }
}
