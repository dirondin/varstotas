// @name Poly Pizza
// @model_source search
// @description Low-poly models of Poly Pizza (poly.pizza; CC0 and CC-BY: the credit goes with the component, File → Export → Attributions). Needs a free API key from poly.pizza/settings/api. Type a search and press Enter.
// @description.ru Низкополигональные модели Poly Pizza (poly.pizza; CC0 и CC-BY: указание автора сохраняется в компоненте, Файл → Экспорт → Авторы). Нужен бесплатный API-ключ с poly.pizza/settings/api. Введите запрос и нажмите Enter.
// @description.lt Poly Pizza žemo poligonų skaičiaus modeliai (poly.pizza; CC0 ir CC-BY: autorystė išsaugoma komponente, Failas → Eksportuoti → Autoriai). Reikia nemokamo API rakto iš poly.pizza/settings/api. Įveskite užklausą ir paspauskite Enter.
// @param api_key:text = "" "API key"
// @param.ru api_key "API-ключ"
// @param.lt api_key "API raktas"
// @notice Models of Poly Pizza (poly.pizza) are CC0 or CC-BY: CC-BY models need a credit to their author (File → Export → Attributions).
// @notice.ru Модели Poly Pizza (poly.pizza) — CC0 или CC-BY: для моделей CC-BY нужно указать автора (Файл → Экспорт → Авторы).
// @notice.lt Poly Pizza (poly.pizza) modeliai – CC0 arba CC-BY: CC-BY modeliams reikia nurodyti autorių (Failas → Eksportuoti → Autoriai).

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
