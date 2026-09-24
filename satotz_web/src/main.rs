use leptos::{ev::SubmitEvent, *};
use satotz_lib::cnf::CNF;
use satotz_lib::solver::Solver;

#[component]
fn App(cx: Scope) -> impl IntoView {
    use leptos::html::Textarea;
    let (name, set_name) = create_signal(cx, "".to_string());
    let input_element: NodeRef<Textarea> = create_node_ref(cx);

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        let cnf = CNF::from_dimacs(&input_element.get().expect("<textarea> to exist").value());
        let mut solver = Solver::from_cnf(cnf);

        if solver.solve() {
            set_name.set("SATISFIABLE".into());
        } else {
            set_name.set("UNSATISFIABLE".into());
        }
    };

    view! { cx,
        <h2>"Input a CNF formula"</h2>
        <form on:submit=on_submit>
            <div style="display: flex">
                <textarea  type="text"
                    value=move || name.get()
                    node_ref=input_element
                    style="height:200px; width: 200px;"
                />
                <input type="submit" value="Solve Formula" style="max-height: 32px; margin-left: 16px;"/>
            </div>
        </form>
        <p>"The formula is: " {name}</p>
    }
}

fn main() {
    mount_to_body(|cx| view! { cx, <App/> })
}
