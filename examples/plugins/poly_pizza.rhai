// @name Poly Pizza
// @model_source search
// @description Low-poly models of Poly Pizza (poly.pizza; CC0 and CC-BY: the credit goes with the component, File → Export → Attributions). Needs a free API key from poly.pizza/settings/api. Type a search and press Enter.
// @param api_key:text = "" "API key"

fn search(query, params) {
    if params.api_key == "" { throw "Set the Poly Pizza API key (free at poly.pizza/settings/api)"; }
    if query == "" { return []; }
    let headers = #{ "x-auth-token": params.api_key };
    let answer = parse_json(fetch_text("https://api.poly.pizza/v1.1/search/" + url_encode(query) + "?Limit=32", headers));
    let out = [];
    for r in answer.results {
        let creator = r.Creator;
        out.push(#{
            id: r.Download,
            name: r.Title,
            category: r.Category,
            tags: r.Tags,
            author: if creator == () { "" } else { creator.Username },
            license: r.Licence,
            thumb: r.Thumbnail,
            page: "https://poly.pizza/m/" + r.ID,
            polycount: r["Tri Count"],
        });
    }
    out
}

// The id is the GLB's URL.
fn download(id, dir, params) {
    #{ url: id }
}
