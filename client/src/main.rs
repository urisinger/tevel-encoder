use anyhow::Result;
use binlayout::epxr::*;
use binlayout::value::Value;
use leptos::task::spawn_local;
use leptos::{ev, prelude::*};
use std::collections::HashMap;

fn main() {
    mount_to_body(|| {
        let (expr, set_expr) = signal(None);

        // Load layout asynchronously
        spawn_local(async move {
            set_expr.set(Some(load_layout().await));
        });

        view! {
            <ErrorBoundary fallback=move |_| {
                view! { <span>"raaaghhh the model failed to loadd"</span> }
            }>
                {move || {
                    expr
                        .with(|expr| {
                            expr
                                .as_ref()
                                .map(|expr| match expr {
                                    Ok(expr) => Ok(view! { <App expr=*expr /> }),
                                    Err(e) => Err(e.clone()),
                                })
                        })
                }}

            </ErrorBoundary>
        }
    });
}

async fn load_layout() -> Result<StoredValue<Expr>, leptos::error::Error> {
    let response = gloo::net::http::Request::get("/layout.lay").send().await?;
    let text = response.text().await?;
    let parsed: Expr = Expr::parse(&text)?;
    Ok(StoredValue::new(parsed))
}

#[component]
fn App(expr: StoredValue<Expr>) -> impl IntoView {
    let (selected_layout, set_selected_layout) = signal::<Option<String>>(None);
    let (form_data, set_form_data) = signal(HashMap::new());
    let (encoded_buffer, set_encoded_buffer) = signal::<Option<String>>(None); // Store encoded buffer

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        let Some(layout_name) = selected_layout.get() else {
            return;
        };
        let expr = expr.read_value();
        let Some(layout) = expr.get_id(&layout_name).and_then(|id| expr.get_type(id)) else {
            return;
        };

        match build_value_from_form(&form_data.get(), layout, &expr, "") {
            Ok(value) => {
                let encoded = value
                    .encode_value()
                    .iter()
                    .map(|b| format!("{:02X} ", b))
                    .collect::<String>();

                set_encoded_buffer.set(Some(encoded));
            }
            Err(e) => set_encoded_buffer.set(Some(e.to_string())),
        }
    };

    // UI Rendering
    view! {
        {move || {
            view! {
                <h1>"Layout Builder"</h1>

                <select on:change=move |ev| {
                    set_selected_layout.set(Some(event_target_value(&ev)))
                }>
                    <option value="">"Select a Layout"</option>
                    {expr
                        .read_value()
                        .layout_ids
                        .keys()
                        .map(|key| {
                            view! { <option value=key.clone()>{key.clone()}</option> }
                        })
                        .collect::<Vec<_>>()}
                </select>

                {{
                    selected_layout
                        .get()
                        .map(|layout_name| {
                            let layout_id = expr.read_value().get_id(&layout_name).unwrap();

                            view! {
                                <form on:submit=on_submit>
                                    <StructBuilder
                                        struct_layout=layout_id
                                        expr=expr
                                        form_data=set_form_data
                                        prefix=StoredValue::new("".to_string())
                                    />
                                    <button type="submit">"Submit"</button>
                                </form>
                            }
                        })
                }}

                // Display the encoded buffer as a hex string
                {move || {
                    encoded_buffer
                        .get()
                        .map(|buffer| {
                            view! {
                                <div>
                                    <h2>"Encoded Buffer:"</h2>
                                    <pre>{buffer}</pre>
                                </div>
                            }
                        })
                }}
            }
        }}
    }
}

// Recursive Struct Builder Component
#[component]
fn StructBuilder(
    struct_layout: LayoutId,
    expr: StoredValue<Expr>,
    form_data: WriteSignal<HashMap<String, ReadSignal<String>>>,
    prefix: StoredValue<String>,
) -> impl IntoView {
    let views = move || {
        expr.read_value()
            .get_type(struct_layout)
            .unwrap()
            .fields
            .iter()
            .map(|(name, ty)| {
                let name = StoredValue::new(if !prefix.read_value().is_empty() {
                    format!("{}.{name}", prefix.read_value())
                } else {
                    name.clone()
                });

                match ty {
                    Type::Struct(inner_id) => view! {
                        <fieldset>
                            <legend>{name.get_value()}</legend>

                            <StructBuilder
                                struct_layout=*inner_id
                                expr=expr
                                form_data=form_data
                                prefix=name
                            />
                        </fieldset>
                    }
                    .into_any(),

                    Type::I8 | Type::I16 | Type::I32 | Type::I64 => {
                        let (value, set_value) = signal("0".to_string());
                        form_data.write().insert(name.get_value(), value);

                        view! {
                            <div>
                                <label>{name.get_value()}</label>
                                <input
                                    type="number"
                                    step="1"
                                    on:input=move |ev| set_value.set(event_target_value(&ev))
                                />
                            </div>
                        }
                        .into_any()
                    }

                    Type::F32 | Type::F64 => {
                        let (value, set_value) = signal("0.0".to_string());
                        form_data.write().insert(name.get_value(), value);

                        view! {
                            <div>
                                <label>{name.get_value()}</label>
                                <input
                                    type="number"
                                    step="any"
                                    on:input=move |ev| set_value.set(event_target_value(&ev))
                                />
                            </div>
                        }
                        .into_any()
                    }
                }
            })
            .collect::<Vec<_>>()
    };

    view! { <div>{views}</div> }
}

// Convert form data into a nested Value struct
fn build_value_from_form(
    form_data: &HashMap<String, ReadSignal<String>>,
    layout: &Struct,
    expr: &Expr,
    prefix: &str,
) -> Result<Value, Error> {
    let mut fields = Vec::new();

    for (name, ty) in &layout.fields {
        let name = if !prefix.is_empty() {
            format!("{prefix}.{name}")
        } else {
            name.clone()
        };
        let data = form_data.get(&name);
        let parsed_value = match ty {
            Type::I8 => Value::I8(data.unwrap().read().parse()?),
            Type::I16 => Value::I16(data.unwrap().read().parse()?),
            Type::I32 => Value::I32(data.unwrap().read().parse()?),
            Type::I64 => Value::I64(data.unwrap().read().parse()?),
            Type::F32 => Value::F32(data.unwrap().read().parse()?),
            Type::F64 => Value::F64(data.unwrap().read().parse()?),
            Type::Struct(inner_name) => {
                let inner_layout = expr.layouts.get(inner_name).unwrap();
                build_value_from_form(form_data, inner_layout, expr, &name)?
            }
        };
        fields.push((name.clone(), parsed_value));
    }

    Ok(Value::Struct { fields })
}
