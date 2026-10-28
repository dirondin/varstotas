# Varstotas sąsajos tekstai: lietuvių kalba.
# Skyriai atitinka raktų sritis (pirmasis rakto žodis). Žr. docs/LOCALIZATION.md.

## ======== Bendra ========

language-name = Lietuvių

# Sistemos failų tvarkyklė, elementams „Rodyti …“ / „Atidaryti … aplanką“.
file-manager = failų tvarkyklė
    .macos = Finder
    .windows = Failų naršyklė

# Parodyti failą sistemos failų tvarkyklėje.
reveal-in-file-manager = Rodyti failų tvarkyklėje
    .macos = Rodyti Finder programoje
    .windows = Rodyti failų naršyklėje

ok = Gerai
cancel = Atšaukti
close = Uždaryti
apply = Taikyti
delete = Ištrinti

## ======== Programa ========

app-edit-dimension-text = Redaguoti matmens tekstą
app-dimension-text-hint = Tuščias laukas atkuria išmatuotą ilgį; `<>` reiškia jį.
# { $step } yra tinklelio žingsnis su vienetu.
app-grid-step = Tinklelio žingsnis { $step }
# { $input } yra įvestas tekstas.
app-invalid-grid-step = Netinkamas tinklelio žingsnis: { $input }
app-invalid-value = Netinkama reikšmė: { $input }
app-undo-move-axes = Perkelti ašis
app-no-credits = Šiame modelyje nėra atsisiųstų komponentų su autorystės nuorodomis
# { $name } yra atšaukiamo veiksmo pavadinimas.
app-flash-undo = Atšaukta: { $name }
app-nothing-to-undo = Nėra ką atšaukti
app-flash-redo = Grąžinta: { $name }
app-nothing-to-redo = Nėra ką grąžinti
app-select-to-group = Pirmiausia pažymėkite, ką sugrupuoti
app-undo-make-group = Sukurti grupę
app-undo-make-component = Sukurti komponentą
app-select-to-explode = Pažymėkite grupę arba komponentą, kurį reikia išskaidyti
app-undo-explode = Išskaidyti
app-undo-hide = Slėpti
app-undo-unhide-all = Rodyti viską
# { $name } yra įrankio pavadinimas.
app-not-implemented = „{ $name }“ dar neįgyvendinta
app-undo-delete = Ištrinti
app-cant-move-into-itself = Grupės ar komponento negalima perkelti į save patį
app-undo-move-out-of-group = Perkelti iš grupės
app-undo-move-into-group = Perkelti į grupę
app-undo-add-scene = Pridėti sceną
app-undo-update-scene = Atnaujinti sceną
app-undo-delete-scene = Ištrinti sceną
app-undo-move-scene = Perkelti sceną
app-undo-cut = Iškirpti
app-no-guides = Pagalbinių linijų nėra
app-snapping-on = Prisitraukimas įjungtas
app-snapping-off = Prisitraukimas išjungtas
app-select-one-component = Pažymėkite vieną komponentą
app-undo-make-unique = Padaryti unikalų
# { $count } yra papildinių skaičius.
app-plugins-loaded = Įkelta papildinių: { $count }
app-select-one-group-or-component = Pažymėkite vieną grupę arba komponentą
app-undo-component-parameters = Komponento parametrai
app-undo-make-parametric = Padaryti parametrinį
app-undo-active-cut = Aktyvus pjūvis
app-undo-reverse-section = Apversti pjūvį
app-undo-soften-edges = Sušvelninti briaunas
app-select-faces-to-reverse = Pažymėkite apverčiamus paviršius
app-undo-reverse-faces = Apversti paviršius
app-select-face-to-orient = Pažymėkite paviršių, pagal kurį bus orientuoti gretimi paviršiai
app-undo-orient-faces = Orientuoti paviršius
app-faces-agree = Visi sujungti paviršiai jau sutampa
# { $count } yra paviršių skaičius.
app-faces-reversed = { $count ->
        [one] Apverstas { $count } paviršius
        [few] Apversti { $count } paviršiai
        [many] Apversta { $count } paviršiaus
       *[other] Apversta { $count } paviršių
    }
app-move-axes-hint = Poslinkis išilgai raudonos, žalios, mėlynos ašių ir neprivalomas pasukimas apie mėlyną ašį: `dx, dy, dz[, kampas]`.
app-undo-reset-axes = Atstatyti ašis
app-select-to-flip = Pažymėkite, ką apversti veidrodiškai
app-undo-flip-red = Apversti išilgai raudonos
app-undo-flip-green = Apversti išilgai žalios
app-undo-flip-blue = Apversti išilgai mėlynos
app-select-to-intersect = Pažymėkite paviršius, grupes ar komponentus, kuriuos reikia susikirsti
app-undo-intersect-model = Sukirsti su modeliu
app-undo-intersect-selection = Sukirsti su žymėjimu
app-undo-intersect-context = Sukirsti su kontekstu
app-no-intersections = Susikirtimų nerasta
# { $count } yra briaunų skaičius.
app-intersections-created = { $count ->
        [one] Sukurta { $count } susikirtimo briauna
        [few] Sukurtos { $count } susikirtimo briaunos
        [many] Sukurta { $count } susikirtimo briaunos
       *[other] Sukurta { $count } susikirtimo briaunų
    }
app-nothing-to-copy = Nepažymėta nieko, ką kopijuoti
# { $error } yra klaidos tekstas.
app-copy-failed = Nukopijuoti nepavyko: { $error }
app-text-no-characters = Tekste nėra matomų simbolių
app-clipboard-empty = Iškarpinė tuščia
# { $factor } yra mastelio koeficientas.
app-resized = Pakeistas dydis ×{ $factor }
app-scene-update = Atnaujinti
app-scene-add = Pridėti
app-scene-delete = Ištrinti
app-scene-move-left = Perkelti į kairę
app-scene-move-right = Perkelti į dešinę
app-pause-animation = Pristabdyti animaciją
app-play-animation = Paleisti animaciją
# { $path } yra failo kelias, { $error } yra klaidos tekstas.
app-cant-use = Negalima naudoti { $path }: { $error }
app-undo-add-material = Pridėti medžiagą
app-dropped-material = Pridėta medžiaga { $name }: spustelėkite sieną, kad ją nudažytumėte
app-no-scenes = Nėra scenų, kurias būtų galima paleisti. Pridėkite scenų skydelyje „Scenos“.
# { $path } yra failo kelias.
app-opened = Atidaryta: { $path }
app-could-not-open = Nepavyko atidaryti { $path }: { $error }
app-saved = Išsaugota: { $path }
app-save-failed = Išsaugoti nepavyko: { $error }
# { $warning } yra įspėjimo tekstas.
app-exported-warning = Eksportuota: { $path }. { $warning }
app-exported = Eksportuota: { $path }
app-export-failed = Eksportuoti nepavyko: { $error }
app-write-denied-windows = Windows uždraudė prieigą. Jei tai aplankas „Dokumentai“, „Paveikslėliai“ ar „Darbalaukis“, Varstotas gali blokuoti Defender kontroliuojama prieiga prie aplankų: Windows sauga → Apsauga nuo virusų ir grėsmių → Apsauga nuo išpirkos reikalaujančių programų → Leisti programai naudotis kontroliuojama prieiga prie aplankų.
# { $format } yra formato pavadinimas.
app-undo-import = Importuoti { $format }
app-imported = Importuota: { $path }
app-autosave-failed = Automatinis išsaugojimas į { $path } nepavyko: { $error }
# { $name } yra failo vardas.
app-recovered = Atkurta { $name } iš automatinio išsaugojimo
app-could-not-recover = Nepavyko atkurti { $name }: { $error }

## Failų dialogų filtrai

app-filter-image = Paveikslėlis
app-filter-script = Scenarijus
app-filter-model = Varstotas modelis

## ======== Meniu ========

menu-intersect-faces = Sukirsti paviršius
menu-flip-along = Apversti išilgai
menu-axes-place = Išdėstyti
menu-axes-move = Perkelti…
menu-axes-reset = Atstatyti
menu-active-cut = Aktyvus pjūvis
menu-reverse = Apversti
menu-align-view = Lygiuoti rodinį
menu-unfold = Išklostyti…
menu-texture = Tekstūra
menu-texture-position = Padėtis
menu-uv-editor = UV redaktorius…
menu-mark-seam = Pažymėti kaip siūlę
menu-clear-seam = Pašalinti siūlę
menu-file = Failas
menu-new-from-template = Naujas iš šablono
menu-open-recent = Atidaryti naujausius
menu-preview-image = Peržiūros vaizdas
menu-preview-image-hint = Paveikslėlis, kurį komponentų biblioteka rodo šiam failui (kitaip nupiešiamas modelis)
menu-import = Importuoti
menu-export = Eksportuoti
menu-unfold-export = Išklotinė (SVG / PDF / DXF)…
menu-edit = Redaguoti
menu-view = Rodinys
menu-face-style = Paviršių stilius
menu-shaded = Su šešėliavimu
menu-shaded-textures = Su šešėliavimu ir tekstūromis
menu-monochrome = Vienspalvis
menu-hidden-line = Paslėptos linijos
menu-wireframe = Vielinis
menu-realistic = Realistiškas
menu-x-ray = Rentgenas
menu-edge-style = Briaunų stilius
menu-edges = Briaunos
menu-edges-hint = Briaunos šiame paviršių stiliuje („Realistiškas“ turi savo jungiklį)
menu-profiles = Profiliai
menu-guides = Pagalbinės linijos
menu-axes = Ašys
menu-ground = Žemė
menu-sky = Dangus
menu-sky-hint = Dangaus gradientas už modelio; išjungta – paprastas šviesus fonas
menu-grid = Tinklelis
menu-shadows = Šešėliai
menu-section-planes = Pjūvio plokštumos
menu-section-cuts = Pjūviai
menu-section-fill = Pjūvio užpildas
menu-viewports = Vaizdo langai
menu-viewports-one = Vienas
menu-viewports-two = Du (greta)
menu-viewports-four = Keturi (2 × 2)
menu-presets = Išankstiniai nustatymai
menu-animation = Animacija
menu-pause = Pristabdyti
menu-play = Paleisti
menu-next-scene = Kita scena
menu-previous-scene = Ankstesnė scena
menu-add-scene = Pridėti sceną
menu-camera = Kamera
menu-standard-views = Standartiniai rodiniai
menu-parallel-projection = Lygiagreti projekcija
menu-perspective = Perspektyva
menu-draw = Braižyti
menu-shapes = Figūros
menu-tools = Įrankiai
menu-solid-tools = Kūnų įrankiai
menu-snapping = Prisitraukimas
menu-extensions = Papildiniai
menu-window = Langas
menu-side-bar = Šoninė juosta
menu-sheets = Lapai…
menu-help = Pagalba
menu-user-guide = Vartotojo vadovas
menu-user-guide-hint = Kaip naudotis Varstotas: įrankiai, meniu, failai, klavišai (F1 virš modelio: aktyvus įrankis)
menu-examples = Pavyzdžiai
menu-examples-hint = Atverti saugyklos pavyzdžius naršyklėje
menu-script-reference = Scenarijų žinynas
menu-script-reference-hint = Scenarijai, generatoriai ir komponentų scenarijai: funkcijos, savybės, pavyzdžiai
menu-licences = Licencijos…
menu-licences-hint = Varstotas licencija, atvirojo kodo komponentai, failų formatai ir internetinės bibliotekos su jų sąlygomis
menu-about = Apie Varstotas
menu-increments = Žingsniai
menu-all-on = Įjungti visus
menu-all-off = Išjungti visus
menu-show-grid = Rodyti tinklelį
menu-increments-move = Perkelti
menu-increments-rotate = Sukti
menu-increments-scale = Keisti mastelį
menu-erase = Trinti
# { $name } yra paskutinio pakeitimo atšaukimo pavadinimas.
menu-undo-named = Atšaukti: { $name }
menu-redo-named = Grąžinti: { $name }

## ======== Komandos ========

cmd-new = Naujas
cmd-open = Atidaryti…
cmd-save = Išsaugoti
cmd-save-as = Išsaugoti kaip…
cmd-preview-show = Rodyti…
cmd-preview-from-file = Iš paveikslėlio failo…
cmd-preview-from-view = Iš dabartinio rodinio
cmd-preview-remove = Pašalinti
cmd-import-obj = OBJ…
cmd-import-stl = STL…
cmd-import-dxf = DXF…
cmd-import-gltf = glTF / GLB…
cmd-mf = 3MF…
cmd-import-dae = COLLADA (DAE)…
cmd-import-fbx = FBX…
cmd-import-ply = PLY…
cmd-import-csv = Taškai (CSV)…
cmd-export-image = 2D grafika (PNG)…
cmd-export-animation = Animacija (GIF / PNG)…
cmd-export-glb = glTF (GLB)…
cmd-export-drawing = 2D brėžinys (SVG / PDF)…
cmd-sheets = Lapai (PDF / SVG)…
cmd-export-report = Ataskaita: dalys ir plotai (CSV)…
cmd-export-faces-json = Paviršiai (JSON)…
cmd-export-attributions = Autorystės nuorodos (atsisiųstų modelių autoriai)…
cmd-run-script = Vykdyti scenarijų…
cmd-quit = Išeiti
cmd-undo = Atšaukti
cmd-redo = Grąžinti
cmd-cut = Iškirpti
cmd-copy = Kopijuoti
cmd-paste = Įklijuoti
cmd-paste-in-place = Įklijuoti į tą pačią vietą
cmd-delete = Ištrinti
cmd-purge-unused = Išvalyti nenaudojamus
cmd-delete-guides = Ištrinti pagalbines linijas
cmd-select-all = Pažymėti viską
cmd-select-none = Nuimti žymėjimą
cmd-hide = Slėpti
cmd-unhide-all = Rodyti viską
cmd-make-group = Sukurti grupę
cmd-make-component = Sukurti komponentą…
cmd-make-unique = Padaryti unikalų
cmd-make-parametric = Padaryti parametrinį
cmd-explode = Išskaidyti
cmd-close-group = Uždaryti grupę
cmd-intersect-model = Su modeliu
cmd-intersect-selection = Su žymėjimu
cmd-intersect-context = Su kontekstu
cmd-reverse-faces = Apversti paviršius
cmd-orient-faces = Orientuoti paviršius
cmd-flip-red = Raudona kryptis
cmd-flip-green = Žalia kryptis
cmd-flip-blue = Mėlyna kryptis
cmd-unfold = Išklostyti…
cmd-physics = Fizika…
cmd-text-3d = 3D tekstas…
cmd-library-3d = Internetinė 3D biblioteka…
cmd-materials-online = Internetinė medžiagų biblioteka…
cmd-zoom-extents = Rodyti visą modelį
cmd-snapping = Prisitraukimas
cmd-show-grid = Rodyti tinklelį
cmd-side-bar = Šoninė juosta
cmd-render = Atvaizdavimas…
cmd-work-planes = Darbo plokštumos…
cmd-model-info = Dokumento informacija…
cmd-component-params = Komponento parametrai…
cmd-generator-editor = Generatorių redaktorius…
cmd-console = Scenarijų konsolė
cmd-log = Žurnalas
cmd-preferences = Nustatymai…

## Aprašai (viena eilutė įrankių juostos paaiškinimui)

cmd-desc-flip = Atspindėti žymėjimą išilgai šios ašies.
cmd-desc-snap = Šį prisitraukimą įjungti / išjungti.
cmd-desc-new = Pradėti naują dokumentą.
cmd-desc-open = Atidaryti Varstotas failą.
cmd-desc-save = Išsaugoti dokumentą.
cmd-desc-save-as = Išsaugoti dokumentą nauju pavadinimu.
cmd-desc-preview-show = Paveikslėlis, kurį komponentų biblioteka rodo šiam failui.
cmd-desc-preview-from-file = Naudoti paveikslėlio failą kaip dokumento peržiūrą.
cmd-desc-preview-from-view = Naudoti dabartinį rodinį kaip dokumento peržiūrą.
cmd-desc-preview-remove = Pašalinti peržiūrą (biblioteka nupiešia modelį).
cmd-desc-run-script = Vykdyti Rhai scenarijaus failą su modeliu.
cmd-desc-quit = Uždaryti Varstotas.
cmd-desc-cut = Nukopijuoti žymėjimą ir jį ištrinti.
cmd-desc-copy = Kopijuoti žymėjimą.
cmd-desc-paste = Pele padėti nukopijuotą geometriją.
cmd-desc-paste-in-place = Įklijuoti ten, kur buvo geometrija.
cmd-desc-delete = Ištrinti žymėjimą.
cmd-desc-purge-unused = Pašalinti medžiagas ir komponentus, kurių modelis nebenaudoja.
cmd-desc-delete-guides = Ištrinti visas pagalbines linijas ir taškus.
cmd-desc-select-all = Pažymėti viską aktyviame kontekste.
cmd-desc-select-none = Išvalyti žymėjimą.
cmd-desc-hide = Paslėpti žymėjimą.
cmd-desc-unhide-all = Parodyti viską, kas paslėpta.
cmd-desc-make-group = Įtraukti žymėjimą į grupę.
cmd-desc-make-component = Paversti žymėjimą komponentu.
cmd-desc-make-unique = Suteikti pažymėtam komponentui savo apibrėžtį: jį redaguojant kitos kopijos nebesikeičia.
cmd-desc-make-parametric = Suteikti pažymėtai grupei parametrus (Komponento parametrai).
cmd-desc-explode = Išskaidyti pažymėtas grupes ir komponentus.
cmd-desc-close-group = Išeiti iš redaguojamos grupės.
cmd-desc-reverse-faces = Sukeisti pažymėtų paviršių priekį ir galą.
cmd-desc-orient-faces = Pasukti gretimus paviršius taip pat kaip pažymėtą.
cmd-desc-unfold = Išdėstyti pažymėtus paviršius plokštumoje (SVG / PDF / DXF).
cmd-desc-physics = Numesti pažymėtas grupes ir komponentus ant modelio veikiant gravitacijai arba nuleisti sieną kaip audinį.
cmd-desc-text-3d = Pridėti tekstą kaip 3D geometriją (grupę).
cmd-desc-zoom-extents = Sutalpinti visą modelį rodinyje.
cmd-desc-snapping = Prisitraukimą įjungti / išjungti.
cmd-desc-show-grid = Tinklelį ant žemės įjungti / išjungti.
cmd-desc-side-bar = Šoninę juostą su kortelėmis („Savybės“, „Medžiagos“ …) rodyti arba slėpti.
cmd-desc-render = Modelio vaizdai, apskaičiuoti sekant spindulius (Langas → Atvaizdavimas).
cmd-desc-work-planes = Išsaugotos braižymo plokštumos.
cmd-desc-model-info = Dokumento statistika, nenaudojamų elementų išvalymas.
cmd-desc-component-params = Pažymėto komponento parametrai: reikšmės, kurios pasuka, pastumia, parodo ar nuspalvina jo dalis.
cmd-desc-library-3d = Nemokami modeliai iš internetinių bibliotekų.
cmd-desc-materials-online = Nemokamos PBR medžiagos iš internetinių bibliotekų.
cmd-desc-uv-editor = Pasirinktų sienų tekstūros koordinačių išdėstymas.
cmd-desc-generator-editor = Rašyti ir vykdyti generatorių scenarijus.
cmd-desc-console = Vykdyti Rhai komandas su modeliu.
cmd-desc-log = Klaidos ir įspėjimai iš visur: scenarijai, importas / eksportas, bibliotekos, atsisiuntimai …
cmd-desc-preferences = Varstotas nustatymai.
# { $name } yra „Atšaukti“ arba „Grąžinti“.
cmd-desc-undo-redo = { $name } paskutinį pakeitimą.
cmd-desc-intersect-model = Pridėti briaunas ten, kur pažymėti paviršiai kerta modelį.
cmd-desc-intersect-selection = Pridėti briaunas ten, kur pažymėti paviršiai kerta vienas kitą.
cmd-desc-intersect-context = Pridėti briaunas ten, kur pažymėti paviršiai kerta redaguojamą grupę.
cmd-desc-solid-outer-shell = Palikti dviejų pažymėtų kūnų išorinį apvalkalą.
cmd-desc-solid-intersect = Palikti bendrą dviejų pažymėtų kūnų tūrį.
cmd-desc-solid-union = Sujungti du pažymėtus kūnus.
cmd-desc-solid-subtract = Iš antrojo iš dviejų pažymėtų kūnų išpjauti pirmąjį.
cmd-desc-solid-trim = Apkirpti antrąjį iš dviejų pažymėtų kūnų pirmuoju.
cmd-desc-solid-split = Padalyti du pažymėtus kūnus į dalis.
# { $card } yra kortelės pavadinimas.
cmd-desc-card = Šoninės juostos kortelę „{ $card }“ rodyti arba slėpti.
# { $name } yra komandos pavadinimas.
cmd-desc-default = { $name }.

cmd-name-preview-show = Peržiūros vaizdas: rodyti

cmd-name-preview-from-file = Peržiūros vaizdas iš failo

cmd-name-preview-from-view = Peržiūros vaizdas iš rodinio

cmd-name-preview-remove = Pašalinti peržiūros vaizdą

cmd-name-intersect-model = Sukirsti su modeliu

cmd-name-intersect-selection = Sukirsti su žymėjimu

cmd-name-intersect-context = Sukirsti su kontekstu

cmd-name-flip-red = Apversti išilgai raudonos

cmd-name-flip-green = Apversti išilgai žalios

cmd-name-flip-blue = Apversti išilgai mėlynos

# { $name } yra formato pavadinimas, kaip meniu.
cmd-name-import = Importuoti { $name }
cmd-name-export = Eksportuoti { $name }
# { $name } yra kūnų operacijos pavadinimas.
cmd-name-solid = Kūnas: { $name }
# { $name } yra prisitraukimo pavadinimas.
cmd-name-snap = Prisitraukimas: { $name }
# { $card } yra kortelės pavadinimas.
cmd-name-card = Šoninė juosta: { $card }

## ======== Įrankiai ========

tool-select = Pažymėti
tool-eraser = Trintukas
tool-paint = Dažų kibiras
tool-texture-position = Tekstūros padėtis
tool-line = Linija
tool-arc = Lankas pagal 2 taškus
tool-arc-center = Lankas
tool-arc3 = Lankas pagal 3 taškus
tool-pie = Skritulio išpjova
tool-bezier = Bezjė kreivė
tool-smooth-curve = Glotni kreivė
tool-freehand = Laisva ranka
tool-edit-curve = Redaguoti kreivę
tool-rectangle = Stačiakampis
tool-rotated-rectangle = Pasuktas stačiakampis
tool-circle = Apskritimas
tool-polygon = Daugiakampis
tool-box = Gretasienis
tool-cylinder = Cilindras
tool-cone = Kūgis
tool-pyramid = Piramidė
tool-sphere = Sfera
tool-torus = Toras
tool-push-pull = Stumti/traukti
tool-move = Perkelti
tool-rotate = Sukti
tool-scale = Keisti mastelį
tool-offset = Atitraukti
tool-chamfer = Nusklembimas
tool-fillet = Užapvalinimas
tool-solid-tools = Kūnų įrankiai
tool-follow-me = Ištempti trajektorija
tool-tape = Matuoklis
tool-protractor = Kampainis
tool-axes = Ašys
tool-dimension = Matmenys
tool-text = Tekstas
tool-section = Pjūvio plokštuma
tool-guide-plane = Pagalbinė plokštuma
tool-slice = Pjaustyti
tool-orbit = Sukimas aplink
tool-pan = Slinkimas
tool-zoom = Mastelis
tool-position-camera = Padėti kamerą
tool-look-around = Apsidairyti
tool-walk = Vaikščioti

tool-select-desc = Pažymėkite objektus, kad juos keistumėte kitais įrankiais.
tool-eraser-desc = Trinti briaunas, pagalbines linijas ir kitus objektus.
tool-paint-desc = Pritaikyti medžiagą paviršiams ir grupėms.
tool-texture-position-desc = Perkelti, sukti ir keisti paviršiaus tekstūros mastelį smeigtukais.
tool-line-desc = Braižyti briaunas. Uždari plokšti kontūrai tampa paviršiais.
tool-arc-desc = Braižyti lanką pagal du galinius taškus ir išlinkį.
tool-arc3-desc = Braižyti lanką per tris taškus.
tool-arc-center-desc = Braižyti lanką pagal centrą ir du taškus.
tool-pie-desc = Braižyti uždarą skritulio išpjovos formos paviršių pagal centrą ir du taškus.
tool-bezier-desc = Braižyti kreivę, formuojamą rankenėlėmis: spustelėkite kampui, nuspauskite ir tempkite glotniam taškui.
tool-smooth-curve-desc = Braižyti glotnią kreivę per spustelėtus taškus; uždara sukuria paviršių.
tool-freehand-desc = Braižyti kreivę laisva ranka: nuspauskite ir tempkite; užbaigta pradžioje, sukuria paviršių.
tool-edit-curve-desc = Pakeisti nubraižytos kreivės formą: tempti jos taškus ir rankenėles, pridėti ar šalinti taškus.
tool-rectangle-desc = Braižyti stačiakampį paviršių pagal du kampus.
tool-rotated-rectangle-desc = Braižyti bet kokiu kampu pasuktą stačiakampį pagal briauną ir plotį.
tool-circle-desc = Braižyti apskritimą pagal centrą ir spindulį.
tool-polygon-desc = Braižyti taisyklingąjį daugiakampį.
tool-box-desc = Braižyti gretasienį: pagrindo kampai, tada aukštis. Jo matmenis galima redaguoti.
tool-cylinder-desc = Braižyti cilindrą: centras, spindulys, aukštis. Jo matmenis galima redaguoti.
tool-cone-desc = Braižyti kūgį: centras, spindulys, aukštis. Jo matmenis galima redaguoti.
tool-pyramid-desc = Braižyti piramidę: pagrindo kampai, tada aukštis. Jos matmenis galima redaguoti.
tool-sphere-desc = Braižyti ant paviršiaus stovinčią sferą: centras, spindulys. Jos matmenis galima redaguoti.
tool-torus-desc = Braižyti ant paviršiaus gulintį torą: centras, spindulys, vamzdžio spindulys. Jo matmenis galima redaguoti.
tool-push-pull-desc = Stumti arba traukti paviršių, kad pridėtumėte ar pašalintumėte tūrį.
tool-move-desc = Perkelti, tempti ar kopijuoti objektus.
tool-rotate-desc = Sukti, tempti ar kopijuoti objektus apie ašį.
tool-scale-desc = Keisti objektų dydį ir tempti juos.
tool-offset-desc = Atitraukti paviršiaus briaunas į vidų arba į išorę.
tool-chamfer-desc = Nusklembti briauną tarp dviejų paviršių plokščia nuoplauša.
tool-fillet-desc = Užapvalinti briauną tarp dviejų paviršių.
tool-solid-tools-desc = Jungti kūnų grupes: išorinis apvalkalas, sankirta, junginys, atimtis, apkirpimas, padalijimas.
tool-follow-me-desc = Stumti paviršių išilgai kelio.
tool-tape-desc = Matuoti atstumus ir kurti pagalbines linijas.
tool-protractor-desc = Matuoti kampus ir kurti pasuktas pagalbines linijas.
tool-axes-desc = Perkelti arba pakeisti braižymo ašių orientaciją.
tool-dimension-desc = Dėti linijinius matmenis tarp taškų arba ant briaunų.
tool-text-desc = Pridėti tekstą su nuoroda, rodančia į modelį.
tool-section-desc = Perpjauti modelį pjūvio plokštuma, kad būtų matyti vidus.
tool-guide-plane-desc = Pridėti darbo plokštumą braižyti ore; ašys ir tinklelis seka paskui ją.
tool-slice-desc = Perpjauti žymėjimą arba modelį plokštuma; pasirinktinai padalyti į dvi grupes.
tool-orbit-desc = Sukti kamerą aplink modelį.
tool-pan-desc = Judinti kamerą į šoną.
tool-zoom-desc = Artinti arba tolinti kamerą.
tool-position-camera-desc = Atsistoti taške akių aukštyje ir žiūrėti į modelį.
tool-look-around-desc = Pasukti galvą: sukti rodinį apie akis.
tool-walk-desc = Vaikščioti po modelį akių aukštyje.

solid-union = Junginys
solid-intersect = Sankirta
solid-subtract = Atimtis
solid-trim = Apkirpti
solid-split = Padalyti
solid-outer-shell = Išorinis apvalkalas

## Pjaustymas

tool-slice-undo = Pjaustyti
tool-slice-undo-split = Pjaustyti ir padalyti
tool-slice-nothing = Pjaustymas: plokštuma nieko neperpjauna
tool-slice-status-first = Spustelėkite pjovimo plokštumos tašką.
tool-slice-status-second = Spustelėkite antrą tašką. Dvigubas spustelėjimas / Enter: paviršiaus plokštuma; rodyklės: normalė išilgai raudonos / žalios / mėlynos / briaunos.
tool-slice-status-third = Spustelėkite trečią tašką, kad perpjautumėte.
# { $step } yra vienas iš tool-slice-status-* tekstų; { $split } yra „on“ arba „off“.
tool-slice-status = { $step } Pjauna žymėjimą arba viską aktyviame kontekste. %COPY%: { $split ->
        [on] Padalijimas ĮJUNGTAS: dalys tampa dviem grupėmis
       *[off] Padalijimas IŠJUNGTAS: pridedamos tik briaunos
    }.

## Matavimų lauko užrašai (bendri)

tool-vcb-measurements = Matavimai
tool-vcb-radius = Spindulys
tool-vcb-angle = Kampas
tool-vcb-length = Ilgis
tool-vcb-segments = Segmentai
tool-vcb-distance = Atstumas
tool-vcb-width-angle = Plotis, kampas
tool-vcb-dimensions = Matmenys
tool-vcb-height = Aukštis
tool-vcb-sides = Kraštinės
tool-vcb-bulge = Išlinkis
tool-vcb-origin = Pradžia
tool-vcb-point = Taškas
tool-vcb-eye-height = Akių aukštis
tool-vcb-height-offset = Aukščio poslinkis
tool-vcb-fov = Matymo laukas

## Lankai, stačiakampiai

tool-undo-pie = Skritulio išpjova
tool-undo-arc = Lankas
tool-undo-arc3 = Lankas pagal 3 taškus
tool-undo-rotated-rectangle = Pasuktas stačiakampis
# { $label } yra figūros pavadinimas (atšaukimo pavadinimas).
tool-degenerate = { $label }: išsigimę duomenys
tool-cursor-angle = Kampas
tool-arc-center-first = Pasirinkite { $what } centrą (paviršius jį orientuoja; rodyklės užfiksuoja plokštumą). Įveskite `{ $segs }s` segmentams.
tool-arc-center-start = Pasirinkite pradžios tašką (spindulį) arba įveskite spindulį.
tool-arc-center-end = Judinkite aplink centrą ir spustelėkite, kad užbaigtumėte { $what }, arba įveskite kampą.
tool-arc-what-pie = skritulio išpjovos
tool-arc-what-arc = lanko
tool-arc3-first = Pasirinkite lanko pradžios tašką.
tool-arc3-second = Pasirinkite antrą tašką, per kurį eina lankas, arba įveskite ilgį.
tool-arc3-third = Pasirinkite galinį tašką arba įveskite lanko kampą.
tool-rotrect-first = Pasirinkite pirmą stačiakampio kampą.
tool-rotrect-second = Pasirinkite antrą kampą (pirmąją briauną) arba įveskite ilgį (`l,w`).
tool-rotrect-third = Judinkite, kad nustatytumėte plotį ir kampą, ir spustelėkite, arba įveskite `plotis` ar `plotis,kampas`.

## Bezjė kreivės

tool-bezier-end = Enter arba dvigubas spustelėjimas užbaigia, spustelėjimas ant pirmo taško uždaro kreivę į paviršių
tool-bezier-freehand-start = Nuspauskite ir tempkite braižydami; užbaikite pradžioje, kad uždarytumėte į paviršių.
tool-bezier-freehand-draw = Tempkite braižydami; atleiskite, kad užbaigtumėte.
tool-bezier-smooth-first = Spustelėkite pirmą kreivės tašką.
# { $end } yra tool-bezier-end tekstas.
tool-bezier-smooth-next = Spustelėkite kitą tašką; { $end }. Backspace pašalina paskutinį tašką.
tool-bezier-handles-first = Spustelėkite pirmą kreivės tašką; nuspauskite ir tempkite, kad ištrauktumėte rankenėles.
tool-bezier-handles-drag = Tempkite, kad suformuotumėte kreivę per šį tašką; atleiskite, kad jį padėtumėte.
# { $end } yra tool-bezier-end tekstas.
tool-bezier-handles-next = Spustelėkite kitą tašką (tempkite rankenėlėms); { $end }. Backspace pašalina paskutinį tašką.

## Kameros įrankiai

tool-position-camera-status = Spustelėkite, kad atsistotumėte taške, arba tempkite iš stovėjimo vietos link žiūrėjimo krypties. Įveskite akių aukštį.
tool-look-around-status = Tempkite, kad apsidairytumėte. Įveskite akių aukštį ir paspauskite Enter.
tool-walk-status = Laikykite mygtuką ir judinkite aukštyn/žemyn, kad eitumėte, kairėn/dešinėn, kad sukumėtės (toliau = greičiau). Shift = aukštyn/žemyn ir į šonus, %CMD% = bėgti. Rodyklių klavišai – žingsnis.

## Užapvalinimas / nusklembimas

tool-undo-fillet = Užapvalinimas
tool-undo-chamfer = Nusklembimas
# { $what } yra operacijos pavadinimas mažosiomis raidėmis, { $error } – priežastis.
tool-chamfer-cant = Šios briaunos negalima { $what }: { $error }
tool-chamfer-what-fillet = užapvalinti
tool-chamfer-what-chamfer = nusklembti
tool-fillet-sizing = Patraukite nuo briaunos, kad nustatytumėte spindulį, spustelėkite, kad baigtumėte. Įveskite spindulį arba `12s`; Esc atšaukia.
tool-chamfer-sizing = Patraukite nuo briaunos, kad nustatytumėte dydį, spustelėkite, kad baigtumėte. Įveskite atstumą; Esc atšaukia.
tool-fillet-hover = Spustelėkite briauną tarp dviejų paviršių, kad ją užapvalintumėte (pažymėtos briaunos: visas); dvigubas spustelėjimas = paskutinis spindulys.
tool-chamfer-hover = Spustelėkite briauną tarp dviejų paviršių, kad ją nusklembtumėte (pažymėtos briaunos: visas); dvigubas spustelėjimas = paskutinis dydis.

## Matuoklis, kampainis, ašys

tool-undo-resize-model = Keisti modelio dydį
tool-undo-resize = Keisti dydį
tool-undo-create-guide = Sukurti pagalbinę liniją
tool-undo-place-axes = Padėti ašis
tool-tape-guides-on = pagalbinės linijos įjungtos
tool-tape-measure-only = tik matuoti
# { $g } yra tool-tape-guides-on arba tool-tape-measure-only.
tool-tape-status-resize = Įveskite naują ilgį, kad pakeistumėte modelio dydį, arba pasirinkite pradžios tašką ({ $g }).
tool-tape-status-start = Pasirinkite pradžios tašką (briauną – lygiagrečiai pagalbinei linijai). %COPY% perjungia pagalbines linijas ({ $g }).
tool-tape-status-end = Pasirinkite galinį tašką arba įveskite atstumą ({ $g }).
tool-protractor-center = Pasirinkite kampainio centrą (rodyklės užfiksuoja jo ašį).
tool-protractor-base = Pasirinkite pagrindinę kryptį.
tool-protractor-angle = Pasirinkite kampą (prisitraukia kas 15°) arba įveskite laipsnius.
tool-axes-origin = Pasirinkite naują pradžią arba įveskite x,y,z.
tool-axes-red = Pasirinkite tašką išilgai naujos raudonos ašies.
tool-axes-green = Pasirinkite tašką naujos žalios ašies kryptimi.

## Kreivių redagavimas, matmenys

tool-undo-add-curve-point = Pridėti kreivės tašką
tool-undo-edit-curve = Redaguoti kreivę
tool-undo-remove-curve-point = Pašalinti kreivės tašką
tool-undo-edit-segments = Redaguoti segmentus
tool-curve-edit-too-few = Redaguoti kreivę: per mažai taškų, kad būtų galima pašalinti vieną
tool-curve-edit-pick = Spustelėkite kreivę, nubraižytą įrankiu „Bezjė kreivė“, „Glotni kreivė“ arba „Laisva ranka“.
tool-curve-edit-drag = Tempkite, kad pakeistumėte formą; atleiskite, kad pritaikytumėte, Esc atšaukia.
tool-curve-edit-insert = Spustelėkite, kad čia pridėtumėte tašką (kreivė išlaiko formą).
tool-curve-edit-idle = Tempkite taškus ir rankenėles; spustelėkite kreivę, kad pridėtumėte tašką, Delete pašalina pažymėtą tašką; dvigubas spustelėjimas ant taško: kampas / glotnus. Enter užbaigia.
tool-undo-dimension = Matmuo
tool-dimension-first = Pasirinkite matmens pradžios tašką arba spustelėkite briauną, kad pažymėtumėte jos ilgį.
tool-dimension-second = Pasirinkite matmens galinį tašką.
tool-dimension-third = Judinkite, kad atitrauktumėte matmens liniją, ir spustelėkite, kad ją padėtumėte.

## Linija, stačiakampis, apskritimas, daugiakampis, lankas

tool-undo-line = Linija
tool-undo-rectangle = Stačiakampis
tool-undo-polygon = Daugiakampis
tool-undo-circle = Apskritimas
tool-line-first = Pasirinkite pradžios tašką. Rodyklių klavišai užfiksuoja ašį.
tool-line-next = Pasirinkite galinį tašką arba įveskite ilgį. Shift = užfiksuoti prisitraukimą, rodyklės = užfiksuoti ašį, Esc/dvigubas spustelėjimas = baigti.
tool-rectangle-first = Pasirinkite pirmą kampą. Rodyklių klavišai parenka plokštumą (→ raudona, ← žalia, ↑ mėlyna normalė).
tool-rectangle-second = Pasirinkite priešingą kampą arba įveskite plotį,aukštį. Esc = atšaukti.
tool-circle-what-polygon = daugiakampio
tool-circle-what-circle = apskritimo
tool-circle-for-sides = kraštinėms
tool-circle-for-segments = segmentams
# { $what } yra tool-circle-what-*, { $kind } yra tool-circle-for-*.
tool-circle-first = Pasirinkite { $what } centrą (po žymekliu esantis paviršius jį orientuoja; rodyklės užfiksuoja normalę). Įveskite `{ $segs }s` { $kind }.
tool-circle-radius = Pasirinkite spindulio tašką arba įveskite spindulį.
tool-arc-first = Pasirinkite lanko pradžios tašką. Pradėkite briaunos gale, kad lankas būtų liečiamas jai.
tool-arc-end-tangent = Pasirinkite galinį tašką (dukart spustelėkite jį žydram liestinės lankui) arba įveskite stygos ilgį.
tool-arc-end = Pasirinkite galinį tašką arba įveskite stygos ilgį.
tool-arc-bulge = Judinkite, kad nustatytumėte išlinkį, ir spustelėkite, arba įveskite išlinkį (`500r` spinduliui). Įveskite `12s` segmentams.

## Trintukas, dažų kibiras, stumti/traukti, atitraukti, ištempti trajektorija

tool-undo-hide = Slėpti
tool-undo-unsoften = Pašalinti sušvelninimą
tool-undo-soften = Sušvelninti
tool-undo-erase = Trinti
tool-eraser-status = Spustelėkite arba tempkite per briaunas, pagalbines linijas, matmenis ar grupes, kad jas ištrintumėte. Shift = slėpti, %CMD% = sušvelninti, %CMD%+Shift = pašalinti sušvelninimą.
tool-paint-mods = Alt = paimti pavyzdį, Ctrl = sujungti paviršiai su ta pačia medžiaga, Shift = visi atitinkantys paviršiai, Ctrl+Shift = atitinkantys sujungti paviršiai
    .macos = Cmd = paimti pavyzdį, Option = sujungti paviršiai su ta pačia medžiaga, Shift = visi atitinkantys paviršiai, Option+Shift = atitinkantys sujungti paviršiai
tool-undo-paint = Dažyti
# { $mods } yra tool-paint-mods.
tool-paint-status = Spustelėkite paviršių ar grupę, kad nudažytumėte. { $mods }.
tool-paint-default-material = Numatytoji
tool-undo-push-pull = Stumti/traukti
# { $keep } yra „on“ arba „off“.
tool-push-pull-moving = Judinkite, kad nustatytumėte atstumą, spustelėkite arba įveskite reikšmę. %COPY% = palikti pradinį paviršių ({ $keep ->
        [on] įjungta
       *[off] išjungta
    }).
tool-push-pull-idle = Spustelėkite paviršių, kad jį stumtumėte ar trauktumėte. Dvigubas spustelėjimas pakartoja paskutinį atstumą.
tool-undo-offset = Atitraukti
tool-offset-moving = Judinkite į vidų arba į išorę ir spustelėkite, arba įveskite atstumą.
tool-offset-idle = Spustelėkite paviršių, kad atitrauktumėte jo briaunas. Dvigubas spustelėjimas pakartoja paskutinį atitraukimą.
tool-undo-follow-me = Ištempti trajektorija
tool-follow-me-profile = Spustelėkite profilio paviršių (iš anksto pažymėkite kelio briaunas, kad iškart būtų išstumta).
tool-follow-me-path = Spustelėkite arba tempkite išilgai kelio briaunų; dvigubas spustelėjimas/Enter baigia. Spustelėkite paviršių, kad stumtumėte aplink jo briauną. Esc = atšaukti.

## Žymėjimas

tool-select-mods = Ctrl = pridėti, Shift = perjungti, Ctrl+Shift = pašalinti
    .macos = Option (arba Cmd) = pridėti, Shift = perjungti, Shift+Option = pašalinti
tool-undo-aim-light = Nukreipti šviesą
tool-undo-active-cut = Aktyvus pjūvis
tool-select-aim = Nukreipkite šviesą: numeskite rankenėlę ten, kur ji turi šviesti. Esc = atšaukti.
# { $mods } yra tool-select-mods.
tool-select-box = Tempkite iš kairės į dešinę – žymėjimas langu, iš dešinės į kairę – žymėjimas kirtimu. { $mods }.
tool-select-idle = Pažymėkite objektus. { $mods }. Tempkite, kad pažymėtumėte kelis. Dukart spustelėkite grupę, kad ją redaguotumėte.

## Perkelti, sukti, keisti mastelį, įklijuoti

tool-undo-copy = Kopijuoti
tool-undo-move = Perkelti
tool-undo-copy-array = Kopijuoti masyvą
tool-undo-rotate = Sukti
tool-undo-rotate-array = Sukti masyvą
tool-undo-scale = Keisti mastelį
tool-undo-paste = Įklijuoti
tool-suffix-copy = (kopija)
tool-suffix-on = (įjungta)
# { $copy } yra tuščias arba " " + tool-suffix-copy.
tool-move-from = Pasirinkite tašką, nuo kurio perkelti{ $copy }. Perkelia žymėjimą arba objektą po žymekliu. %COPY% = kopijuoti; po kopijavimo įveskite `5x` arba `/5` masyvams.
tool-move-to = Pasirinkite paskirties tašką arba įveskite atstumą{ $copy }. Shift = užfiksuoti prisitraukimą, rodyklės = užfiksuoti ašį.
tool-rotate-center = Pasirinkite sukimo centrą (paviršius orientuoja kampainį; rodyklės užfiksuoja jo ašį).
tool-rotate-zero = Pasirinkite nulinę kryptį.
# { $step } yra tool-rotate-rounded arba tool-rotate-snaps; { $on } yra tuščias arba " " + tool-suffix-on.
tool-rotate-angle = Pasirinkite kampą ({ $step }) arba įveskite laipsnius. %COPY% = kopijuoti{ $on }; po kopijavimo įveskite `6x` radialiniam masyvui.
# { $step } yra kampas.
tool-rotate-rounded = suapvalinta iki { $step }°
tool-rotate-snaps = prisitraukia kas 15°
tool-scale-uniformly = tolygiai
tool-scale-two-axes = išilgai dviejų ašių
tool-scale-one-axis = išilgai vienos ašies
tool-scale-about-centre = centro
tool-scale-about-opposite = priešingos rankenėlės
# { $what } yra tool-scale-uniformly / -two-axes / -one-axis, { $about } yra tool-scale-about-*.
tool-scale-dragging = Judinkite, kad keistumėte mastelį { $what } pagal { $about }; spustelėkite arba įveskite koeficientą (kiekvienai ašiai: `1.5; 2`). %CMD% = pagal centrą.
tool-scale-select = Pažymėkite objektus (arba užveskite ant vieno), tada tempkite rankenėlę.
tool-scale-idle = Tempkite rankenėlę: kampai keičia mastelį tolygiai, briaunų vidurio taškai – dviem ašimis, paviršių centrai – viena ašimi. %CMD% = pagal centrą.
# { $factors } yra mastelio koeficientas (-ai).
tool-scale-tooltip-uniform = Tolygus mastelis { $factors }
tool-scale-tooltip = Mastelis { $factors }
# { $name } yra dedamo komponento pavadinimas.
tool-paste-place-named = Spustelėkite, kad padėtumėte { $name } (Esc atšaukia).
# { $count } yra objektų skaičius.
tool-paste-place = Spustelėkite, kad padėtumėte įklijuot{ $count ->
        [one] ą { $count } objektą
        [few] us { $count } objektus
        [many] o { $count } objekto
       *[other] ų { $count } objektų
    } (Esc atšaukia).

## Pjūvis, kūnai, tekstas, tekstūra, darbo plokštuma, įvairūs

tool-undo-section-plane = Pjūvio plokštuma
tool-section-status = Spustelėkite paviršių, kad padėtumėte pjūvio plokštumą. Ji nupjauna pusę už savo rodyklių.

## Figūros (gretasienis, cilindras, kūgis, piramidė, sfera, toras)

tool-vcb-tube = Vamzdis
# { $what } yra figūros pavadinimas mažosiomis raidėmis.
tool-prim-box-first = Spustelėkite pirmą { $what } pagrindo kampą (po žymekliu esantis paviršius jį orientuoja; rodyklės nustato plokštumą).
tool-prim-round-first = Spustelėkite { $what } pagrindo centrą (po žymekliu esantis paviršius jį orientuoja). Įveskite `{ $segs }s` segmentams.
tool-prim-box-second = Spustelėkite priešingą kampą arba įveskite `plotis, gylis` (`plotis, gylis, aukštis` padeda iškart).
tool-prim-sphere-radius = Judinkite, kad nustatytumėte spindulį, ir spustelėkite, arba įveskite jį.
tool-prim-torus-radius = Spustelėkite spindulį (iki vamzdžio vidurio) arba įveskite `spindulys, vamzdis`.
tool-prim-round-radius = Spustelėkite spindulį arba įveskite `spindulys` (`spindulys, aukštis` padeda iškart).
tool-prim-torus-tube = Judinkite, kad nustatytumėte vamzdžio spindulį, ir spustelėkite, arba įveskite jį.
tool-prim-height = Judinkite aukštyn ar žemyn, kad nustatytumėte aukštį, ir spustelėkite, arba įveskite jį.

# { $name } yra kūnų operacijos pavadinimas.
tool-solid-not-solid = { $name }: ne kūnas (kiekviena briauna turi riboti lygiai du paviršius, jokių įdėtų grupių).
tool-solid-first = { $name }: spustelėkite pirmą kūno grupę ar komponentą.
tool-solid-next = { $name }: spustelėkite kitą kūną, kad jį pridėtumėte (Esc – pradėti iš naujo).
tool-solid-cut = { $name }: spustelėkite kūną, kurį reikia išpjauti pirmuoju (Esc – pradėti iš naujo).
tool-solid-second = { $name }: spustelėkite antrą kūną (Esc – pradėti iš naujo).
tool-undo-text = Tekstas
tool-text-first = Spustelėkite tašką, briauną ar paviršių, kad pritvirtintumėte tekstą, arba tuščią vietą – tekstui be nuorodos.
tool-text-place = Judinkite, kad padėtumėte tekstą, ir spustelėkite. Redaguokite tekstą kortelėje „Savybės“.
tool-text3d-status = Spustelėkite, kad padėtumėte 3D tekstą (Esc atšaukia).
tool-undo-texture-position = Tekstūros padėtis
tool-texture-pick = Spustelėkite paviršių su tekstūra, kad nustatytumėte jos padėtį.
tool-texture-move = Tekstūra perkeliama. Esc atšaukia.
tool-texture-green = Tekstūra sukama ir keičiamas jos mastelis apie raudoną smeigtuką.
tool-texture-blue = Keičiamas tekstūros aukštis.
tool-texture-idle = Tempkite paviršių arba raudoną smeigtuką, kad perkeltumėte, žalią – kad sukti/keisti mastelį, mėlyną – kad ištempti. Baigę paspauskite Enter arba spustelėkite šalia.
tool-texture-idle-many = Paviršių: { $count }, tekstūra tęsiama nuo pasirinkto. Shift+spustelėjimas prideda arba pašalina paviršių, Alt+spustelėjimas pratęsia tekstūrą ant kito.
tool-texture-cant-continue = Tekstūros negalima pratęsti ant šio paviršiaus (jis statmenas).
tool-texture-vcb = Plytelė

tool-undo-add-guide-plane = Pridėti pagalbinę plokštumą
tool-undo-edit-guide-plane = Redaguoti pagalbinę plokštumą
tool-workplane-origin = Spustelėkite plokštumos pradžią (arba įveskite x,y,z). Tempkite aktyvios plokštumos rankenėles: pradžią, normalės galą, raudoną sukimo rankenėlę.
tool-workplane-red = Spustelėkite tašką išilgai naujos raudonos ašies. Dvigubas spustelėjimas / Enter: paviršiaus plokštuma; rodyklės: normalė išilgai raudonos / žalios / mėlynos / briaunos.
tool-workplane-third = Spustelėkite trečią plokštumos tašką.
tool-workplane-drag-origin = Perkelti plokštumos pradžią.
tool-workplane-drag-normal = Pakreipti plokštumą: judinti jos normalę.
tool-workplane-drag-spin = Sukti plokštumą apie jos normalę.
tool-zoom-status = Tempkite aukštyn/žemyn, kad keistumėte mastelį. Įveskite matymo lauką ir paspauskite Enter.
tool-orbit-status = Tempkite, kad sukumėtės aplink. Shift = slinkti.
tool-pan-status = Tempkite, kad slinktumėte.
# { $prompt } yra papildinio pranešimas.
tool-plugin-select = { $prompt } – spustelėkite, kad pridėtumėte / pašalintumėte, Enter – tęsti, Esc atšaukia.
# { $name } yra įrankio pavadinimas.
tool-not-implemented = Įrankis „{ $name }“ dar neįgyvendintas.

## Patarimų skydelis (ilga pagalba kiekvienam įrankiui; eilutė, prasidedanti •, yra sąrašo elementas)

tool-select-instructor =
    Spustelėkite objektą, kad jį pažymėtumėte.
    • Dvigubas spustelėjimas ant paviršiaus: pažymi jį su briaunomis.
    • Trigubas spustelėjimas: pažymi viską, kas sujungta.
    • Tempimas iš kairės į dešinę: žymėjimas langu (visiškai viduje).
    • Tempimas iš dešinės į kairę: žymėjimas kirtimu (liečiantys).
    • Ctrl = pridėti, Shift = perjungti, Ctrl+Shift = pašalinti.
    • Dukart spustelėkite grupę, kad ją redaguotumėte; Esc ją uždaro.
    • Delete ištrina žymėjimą.
    .macos =
        Spustelėkite objektą, kad jį pažymėtumėte.
        • Dvigubas spustelėjimas ant paviršiaus: pažymi jį su briaunomis.
        • Trigubas spustelėjimas: pažymi viską, kas sujungta.
        • Tempimas iš kairės į dešinę: žymėjimas langu (visiškai viduje).
        • Tempimas iš dešinės į kairę: žymėjimas kirtimu (liečiantys).
        • Option (arba Cmd) = pridėti, Shift = perjungti, Shift+Option = pašalinti.
        • Dukart spustelėkite grupę, kad ją redaguotumėte; Esc ją uždaro.
        • Delete ištrina žymėjimą.
tool-eraser-instructor = Spustelėkite arba tempkite per briaunas, kad jas ištrintumėte. Ištrynus briauną pašalinami ją ribojantys paviršiai.
tool-paint-instructor =
    Pasirinkite medžiagą skydelyje „Medžiagos“, tada spustelėkite paviršius.
    • Alt: paimti medžiagos pavyzdį.
    • Ctrl: užpildyti sujungtus paviršius ta pačia medžiaga.
    • Shift: pakeisti medžiagą visuose atitinkančiuose paviršiuose.
    • Ctrl+Shift: pakeisti ją atitinkančiuose sujungtuose paviršiuose.
    .macos =
        Pasirinkite medžiagą skydelyje „Medžiagos“, tada spustelėkite paviršius.
        • Cmd: paimti medžiagos pavyzdį.
        • Option: užpildyti sujungtus paviršius ta pačia medžiaga.
        • Shift: pakeisti medžiagą visuose atitinkančiuose paviršiuose.
        • Option+Shift: pakeisti ją atitinkančiuose sujungtuose paviršiuose.
tool-texture-position-instructor =
    Nustatykite paviršiaus su tekstūra tekstūros padėtį (pažymėkite jį arba spustelėkite).
    • Tempkite paviršių arba raudoną smeigtuką: perkelia tekstūrą.
    • Žalias smeigtukas: sukti ir keisti mastelį apie raudoną smeigtuką.
    • Mėlynas smeigtukas: ištempti tekstūros aukštį.
    • Smeigtukai prilimpa prie viršūnių ir kraštinių; dvigubas spustelėjimas ant kraštinės išlygiuoja tekstūrą pagal ją.
    • Shift+spustelėjimas prideda paviršių, Alt+spustelėjimas pratęsia ant jo tekstūrą (per kampus).
    • Įveskite plytelės dydį paviršiuje (`800` arba `800;400`) arba kampą (`30°`).
    • Esc atšaukia tempimą; Enter arba spustelėjimas šalia baigia.
tool-line-instructor =
    Spustelėkite pradžiai, spustelėkite pabaigai. Kiekvienas galas pradeda naują atkarpą.
    Įveskite ilgį ir paspauskite Enter tikslumui.
    Shift užfiksuoja prisitraukimo kryptį; rodyklių klavišai užfiksuoja ašis.
tool-arc-instructor =
    Spustelėkite pradžios tašką, spustelėkite galinį tašką, tada judinkite, kad nustatytumėte išlinkį, ir spustelėkite.
    • Įveskite stygos ilgį, tada išlinkį (arba `500r` spinduliui); `12s` nustato segmentus.
    • Pradėjus briaunos gale, žydras lankas yra jai liestinis.
    • Dvigubas spustelėjimas nubraižo lanką, liestinį ankstesniam.
tool-arc-center-instructor =
    Spustelėkite centrą, spustelėkite pradžios tašką (spindulį), tada judinkite aplink ir spustelėkite pabaigą.
    Po pirmo spustelėjimo įveskite spindulį, tada kampą. `12s` nustato segmentus.
tool-arc3-instructor =
    Spustelėkite pradžios tašką, spustelėkite tašką, per kurį eina lankas, tada spustelėkite pabaigą.
    Po pirmo spustelėjimo įveskite ilgį, po antro – kampą.
tool-bezier-instructor =
    Spustelėkite kreivės taškus; nuspauskite ir tempkite ties tašku, kad ištrauktumėte jo rankenėles (paprastas spustelėjimas sukuria kampą).
    Enter arba dvigubas spustelėjimas užbaigia atvirą kreivę (tik briaunos); spustelėjimas ant pirmo taško uždaro ją į paviršių.
    Backspace pašalina paskutinį tašką. Įveskite ilgį kitam taškui žymeklio kryptimi; `12s` nustato segmentus kiekvienam ruožui.
tool-edit-curve-instructor =
    Pakeiskite kreivės, nubraižytos įrankiu „Bezjė kreivė“, „Glotni kreivė“ ar „Laisva ranka“, formą (dukart spustelėkite ją įrankiu „Pažymėti“ arba pažymėkite ir pasirinkite „Redaguoti kreivę“).
    • Tempkite tašką (kvadratą), kad jį perkeltumėte, rankenėlę (apskritimą) – kad sulenktumėte kreivę; glotnus taškas išlaiko savo rankenėles vienoje linijoje.
    • Spustelėkite kreivę tarp taškų, kad pridėtumėte tašką; Delete pašalina pažymėtą tašką.
    • Dvigubas spustelėjimas ant taško: kampas ↔ glotnus. `12s` nustato segmentus kiekvienam ruožui.
    • Enter, Esc arba spustelėjimas tuščioje vietoje baigia.
tool-smooth-curve-instructor =
    Spustelėkite taškus; kreivė eina glotniai per juos.
    Enter arba dvigubas spustelėjimas užbaigia atvirą kreivę (tik briaunos); spustelėjimas ant pirmo taško uždaro ją į paviršių.
    Backspace pašalina paskutinį tašką. Įveskite ilgį kitam taškui žymeklio kryptimi; `12s` nustato segmentus kiekvienam ruožui.
tool-freehand-instructor =
    Nuspauskite ir tempkite braižydami; atleiskite, kad užbaigtumėte. Brūkšnys išlyginamas.
    Brūkšnys, pasibaigiantis pradžioje, užsidaro į paviršių; kitaip lieka briaunos.
tool-pie-instructor =
    Spustelėkite centrą, spustelėkite pradžios tašką (spindulį), tada spustelėkite pabaigą, kad sukurtumėte uždarą skritulio išpjovos paviršių.
    Įveskite spindulį, tada kampą.
tool-rectangle-instructor = Spustelėkite pirmą kampą, tada priešingą kampą. Įveskite plotį,aukštį tikslumui.
tool-rotated-rectangle-instructor =
    Spustelėkite pirmą kampą, spustelėkite antrą (pirmąją briauną), tada judinkite, kad nustatytumėte plotį, ir spustelėkite.
    Įveskite briaunos ilgį (arba `l,w`), tada plotį (arba `w,kampas`).
tool-circle-instructor = Spustelėkite centrą, tada judinkite ir spustelėkite, kad nustatytumėte spindulį. Prieš spustelėdami įveskite `24s`, kad nustatytumėte segmentus.
tool-polygon-instructor = Spustelėkite centrą, tada spindulį. Įveskite `6s`, kad nustatytumėte kraštinių skaičių.
tool-box-instructor =
    Spustelėkite pirmą pagrindo kampą, spustelėkite priešingą kampą, tada judinkite aukštyn ar žemyn ir spustelėkite, kad nustatytumėte aukštį.
    • Po žymekliu esantis paviršius laiko pagrindą; po pirmo spustelėjimo rodyklės nustato jo plokštumą.
    • Įveskite `plotis, gylis`, tada aukštį – arba iškart `plotis, gylis, aukštis`.
    • Rezultatas yra komponentas: keiskite jo matmenis kortelėje „Savybės“; išskaidykite jį, kad gautumėte paprastą geometriją.
tool-cylinder-instructor =
    Spustelėkite pagrindo centrą, spustelėkite spindulį, tada judinkite aukštyn ar žemyn ir spustelėkite, kad nustatytumėte aukštį.
    • Įveskite spindulį, tada aukštį – arba iškart `spindulys, aukštis`; `32s` nustato segmentus.
    • Rezultatas yra komponentas: keiskite jo matmenis kortelėje „Savybės“; išskaidykite jį, kad gautumėte paprastą geometriją.
tool-sphere-instructor =
    Spustelėkite ten, kur sfera stovi, tada judinkite ir spustelėkite, kad nustatytumėte spindulį (arba jį įveskite).
    • `32s` nustato segmentus. Rezultatas yra komponentas: keiskite jo matmenis kortelėje „Savybės“.
tool-torus-instructor =
    Spustelėkite centrą, spustelėkite spindulį (iki vamzdžio vidurio), tada judinkite ir spustelėkite, kad nustatytumėte vamzdį.
    • Iškart įveskite `spindulys, vamzdis`; `32s` nustato segmentus.
    • Rezultatas yra komponentas: keiskite jo matmenis kortelėje „Savybės“.
tool-push-pull-instructor =
    Užveskite ant paviršiaus, spustelėkite, judinkite, kad nustatytumėte atstumą, spustelėkite dar kartą.
    %COPY% palieka pradinį paviršių. Dvigubas spustelėjimas pakartoja paskutinį atstumą.
tool-move-instructor = Spustelėkite tašką, kad paimtumėte žymėjimą, spustelėkite paskirties tašką. %COPY% sukuria kopiją; įveskite `5x` masyvui.
tool-rotate-instructor = Spustelėkite centrą, spustelėkite nulinę kryptį, spustelėkite kampą. Įveskite kampą laipsniais.
tool-scale-instructor = Pažymėkite objektus, tada tempkite rankenėlę. Įveskite mastelio koeficientą.
tool-offset-instructor = Spustelėkite paviršių, judinkite į vidų ar į išorę, spustelėkite. Įveskite atstumą.
tool-chamfer-instructor =
    Spustelėkite briauną tarp dviejų paviršių (arba vieną iš pažymėtų briaunų: visas), patraukite nuo jos, kad nustatytumėte dydį, spustelėkite dar kartą.
    • Dydžio nustatymo metu įveskite atstumą; dvigubas spustelėjimas naudoja paskutinį dydį.
    • Įvedus iškart po nusklembimo, jis perdaromas nauju dydžiu. Esc atšaukia.
tool-fillet-instructor =
    Spustelėkite briauną tarp dviejų paviršių (arba vieną iš pažymėtų briaunų: visas), patraukite nuo jos, kad nustatytumėte spindulį, spustelėkite dar kartą.
    • Įveskite spindulį, `12s` segmentų skaičiui arba abu: `20,12s`.
    • Dvigubas spustelėjimas naudoja paskutinį spindulį; įvedus iškart po užapvalinimo, jis perdaromas. Esc atšaukia.
tool-solid-tools-instructor =
    Įrankiai → Kūnų įrankiai: pasirinkite operaciją, tada spustelėkite pirmą kūno grupę, paskui antrą.
    • Kai kūnai pažymėti, operacija vykdoma iškart.
    • Atimtis / Apkirpimas išpjauna antrą kūną pirmuoju (Apkirpimas palieka pirmąjį).
    • Padalijimas sukuria tris dalis; Išorinis apvalkalas / Junginys toliau prideda spustelėtus kūnus.
    • Kūnas: kiekviena briauna riboja lygiai du paviršius, jokių įdėtų grupių.
tool-follow-me-instructor = Pažymėkite kelią, tada spustelėkite profilio paviršių, kad jį išstumtumėte.
tool-tape-instructor = Spustelėkite du taškus, kad išmatuotumėte. Pradėjus ant briaunos, sukuriama lygiagreti pagalbinė linija.
tool-protractor-instructor = Spustelėkite centrą, spustelėkite pagrindinę kryptį, spustelėkite kampą, kad sukurtumėte pagalbinę liniją.
tool-axes-instructor =
    Spustelėkite naują pradžią, spustelėkite tašką išilgai raudonos ašies, tada tašką žalios ašies kryptimi.
    Įveskite `x,y,z` pradžiai. Spustelėkite ašis dešiniuoju pelės mygtuku: Išdėstyti / Perkelti / Atstatyti.
tool-dimension-instructor =
    Spustelėkite pradžios ir galinį tašką (arba spustelėkite briauną), tada judinkite, kad atitrauktumėte matmenį, ir spustelėkite.
    Dukart spustelėkite matmens tekstą įrankiu „Pažymėti“, kad jį redaguotumėte; tuščias tekstas atkuria išmatuotą ilgį.
tool-text-instructor =
    Spustelėkite tašką, briauną ar paviršių, patraukite tekstą ir spustelėkite dar kartą.
    • Numatytasis tekstas: taško koordinatės, briaunos ilgis, paviršiaus plotas.
    • Spustelėkite tuščią vietą – tekstui be nuorodos.
    • Redaguokite tekstą kortelėje „Savybės“; perkelkite jį įrankiu „Perkelti“ arba perkeldami geometriją.
tool-section-instructor =
    Spustelėkite paviršių, kad ant jo padėtumėte pjūvio plokštumą; ji tampa aktyviu pjūviu.
    • Rodyklės rodo dalį, kuri lieka matoma.
    • Pažymėkite plokštumą pagal jos kraštą; perkelkite ją įrankiu „Perkelti“, ištrinkite klavišu Delete.
    • Dešinysis pelės mygtukas ant plokštumos: Aktyvus pjūvis, Apversti, Lygiuoti rodinį.
    • Rodinys → Pjūvio plokštumos / Pjūviai / Pjūvio užpildas.
tool-guide-plane-instructor =
    Spustelėkite tris taškus: pradžią, tašką išilgai raudonos, tašką plokštumoje.
    • Dvigubas spustelėjimas ant paviršiaus (arba Enter po pirmo spustelėjimo): to paviršiaus plokštuma.
    • Po pirmo spustelėjimo rodyklės nustato normalę: → raudona, ← žalia, ↑ mėlyna, ↓ paskutinė briauna.
    • Nauja plokštuma tampa aktyvia: taškai ore krenta ant jos, ašys ir tinklelis seka paskui ją.
    • Tempkite aktyvios plokštumos rankenėles: pradžią, normalės galą (pakreipti), raudoną rankenėlę (sukti).
    • Langas → Darbo plokštumos: aktyvuoti, slėpti, pervadinti, ištrinti ar įvesti kampus.
tool-slice-instructor =
    Nustatykite pjovimo plokštumą kaip pagalbinę plokštumą: spustelėkite tris taškus.
    • Dvigubas spustelėjimas ant paviršiaus (arba Enter po pirmo spustelėjimo): to paviršiaus plokštuma.
    • Po pirmo spustelėjimo rodyklės nustato normalę: → raudona, ← žalia, ↑ mėlyna, ↓ paskutinė briauna.
    • Pjauna žymėjimą (paviršius, grupes, komponentus) arba viską aktyviame kontekste.
    • Briaunos pridedamos išilgai pjūvio; komponentas, bendras su kitomis kopijomis, pirmiausia padaromas unikalus.
    • %COPY% perjungia padalijimą (+ prie žymeklio): kiekvienas perpjautas kūnas tampa dviem grupėmis, uždaromis pjūvio vietoje.
tool-orbit-instructor =
    Tempkite, kad sukumėtės aplink tašką po žymekliu.
    Vidurinis pelės mygtukas sukasi aplink bet kuriuo įrankiu; Shift+vidurinis slenka.
    Jutiklinis kilimėlis: slinkimas dviem pirštais sukasi aplink, Shift+slinkimas slenka, suspaudimas keičia mastelį.
tool-pan-instructor = Tempkite, kad slinktumėte rodinį. Shift + vidurinis pelės mygtukas slenka bet kuriuo įrankiu.
tool-zoom-instructor =
    Tempkite aukštyn, kad priartintumėte, žemyn – kad nutolintumėte. Pelės ratukas artina žymeklio link bet kuriuo įrankiu.
    Įveskite matymo lauką (pvz., 35) ir paspauskite Enter. Shift+Z pritaiko visą modelį.
tool-position-camera-instructor =
    Spustelėkite, kad atsistotumėte taške ir žiūrėtumėte ten, kur dabar žiūri kamera.
    • Tempkite iš stovėjimo vietos link žiūrėjimo krypties.
    • Prieš spustelėdami įveskite akių aukštį (numatytasis 1676 mm).
    • Vėliau persijungia į „Apsidairyti“.
tool-look-around-instructor = Tempkite, kad pasuktumėte rodinį apie akis. Įveskite akių aukštį ir paspauskite Enter.
tool-walk-instructor =
    Tempkite aukštyn/žemyn, kad eitumėte pirmyn/atgal, kairėn/dešinėn, kad sukumėtės.
    • Shift: judėti aukštyn/žemyn ir į šonus. %CMD%: bėgti.
    • Rodyklių klavišai: žingsnis ir sukimasis. Įveskite akių aukštį ir paspauskite Enter.

# { $prompt } yra papildinio pranešimas, { $index } iš { $total } – renkamas taškas.
tool-plugin-pick = { $prompt } ({ $index } iš { $total }; Esc atšaukia).

## ======== Prisitraukimas ir nuorodos ========

snap-endpoint = Galiniai taškai
snap-midpoint = Vidurio taškai
snap-center = Centrai
snap-intersection = Sankirtos
snap-origin = Pradžia
snap-on-edge = Ant briaunos
snap-on-face = Ant paviršiaus
snap-on-guide = Ant pagalbinių linijų
snap-guide-point = Pagalbiniai taškai
snap-on-red-axis = Raudona ašis
snap-on-green-axis = Žalia ašis
snap-on-blue-axis = Mėlyna ašis
snap-parallel-to-edge = Lygiagrečiai briaunai
snap-perpendicular-to-edge = Statmenai briaunai
snap-from-point = Nuo taško
snap-tangent = Liestinė viršūnėje
snap-square = Kvadratas
snap-golden-section = Auksinis pjūvis
snap-half-circle = Pusapskritimis
snap-grid-point = Tinklelio taškai
snap-on-grid-line = Tinklelio linijos

inference-endpoint = Galinis taškas
inference-midpoint = Vidurio taškas
inference-center = Centras
inference-intersection = Sankirta
inference-on-edge = Ant briaunos
inference-on-red-axis = Ant raudonos ašies
inference-on-green-axis = Ant žalios ašies
inference-on-blue-axis = Ant mėlynos ašies
inference-parallel-to-edge = Lygiagrečiai briaunai
inference-perpendicular-to-edge = Statmenai briaunai
inference-from-point = Nuo taško
inference-on-face = Ant paviršiaus
inference-guide-point = Pagalbinis taškas
inference-on-guide = Ant linijos
inference-origin = Pradžia
inference-square = Kvadratas
inference-golden-section = Auksinis pjūvis
inference-half-circle = Pusapskritimis
inference-tangent = Liestinė viršūnėje
inference-grid-point = Tinklelio taškas
inference-on-grid-line = Ant tinklelio linijos

## Prisitraukimo meniu grupės

snap-group-points = Taškai
snap-group-on-objects = Ant objektų
snap-group-directions = Kryptys
snap-group-shapes = Figūros
snap-group-grid = Tinklelis

## ======== Redagavimas ========

edit-nothing-selected = nieko nepasirinkta
edit-nothing-to-copy = nėra ką kopijuoti
edit-component-file-empty = komponento failas tuščias
edit-internal-error = vidinė klaida
edit-untitled = Be pavadinimo
# { $error } is the error text.
edit-parameters-error = Parametrai: { $error }
edit-undo-erase = Trinti
edit-undo-delete-guides = Naikinti pagalbines linijas

# { $label } is the undo name of the operation, { $error } the reason.
edit-failed = { $label } nepavyko: { $error }
edit-op-error = { $label }: { $error }

## ======== Rodiniai ========

view-iso = Izometrija
view-top = Viršus
view-bottom = Apačia
view-front = Priekis
view-back = Galas
view-left = Kairė
view-right = Dešinė

## View HUD (the viewport's drop-downs)

view-style-shaded = Šešėliuotas
view-style-textured = Su tekstūromis
view-style-textured-menu = Šešėliuotas su tekstūromis
view-style-monochrome = Vienspalvis
view-style-hidden-line = Paslėptos linijos
view-style-wireframe = Karkasas
view-style-realistic = Realistinis
# $label is the style name.
view-style-xray-suffix = { $label } · Rentgenas
view-parallel = Lygiagretus
view-perspective = Perspektyvinis
# $view is a standard view name, $projection is Parallel / Perspective.
view-label = { $view } · { $projection }
view-parallel-projection = Lygiagreti projekcija
view-fov = Matymo laukas
view-fov-tip = Vertikalus kampas, kurį mato perspektyvinė kamera (galima įvesti ir įrankyje „Artinti“)
view-zoom-extents = Rodyti viską
view-viewports = Vaizdo sritys
view-panes-1 = Vienas rodinys
view-panes-2 = Du rodiniai šalia
view-panes-4 = Keturi rodiniai (2 × 2)
view-presets = Vaizdo sričių šablonai
# $label is the current view, e.g. "Top · Parallel".
view-hud-view-tip = Rodinys: { $label } — standartiniai rodiniai, perspektyvinė ar lygiagreti projekcija, matymo laukas, vaizdo sritys
# $label is the current style.
view-hud-style-tip = Stilius: { $label } — kaip piešiami paviršiai ir briaunos (taip pat Rodinys → Paviršių stilius)
view-xray = Rentgenas
view-edges = Briaunos
view-profiles = Kontūrai
view-show-axes = Ašys
view-show-ground = Žemė
view-show-sky = Dangus
view-show-grid = Tinklelis
view-show-guides = Pagalbinės linijos
view-show-shadows = Šešėliai
view-show-section-planes = Pjūvio plokštumos
view-show-section-cuts = Pjūviai
view-show-section-fill = Pjūvio užpildas
view-show-model-header = Modelis (visi rodiniai)

## Viewport presets

view-preset-front-iso = Priekis + izometrija
view-preset-top-iso = Viršus + izometrija
view-preset-front-right = Priekis + dešinė
view-preset-front-left-top-iso = Priekis, kairė, viršus + izometrija
view-preset-top-front-right-iso = Viršus, priekis, dešinė + izometrija
view-preset-front-back-left-right = Priekis, galas, kairė, dešinė

## Viewport messages

view-nothing-rendered = nieko nesugeneruota
view-no-renderer = 3D vaizdavimo variklis nepasiekiamas
# $error is the reason.
view-renderer-unavailable-yet = 3D vaizdavimo variklis dar nepasiekiamas ({ $error })
view-renderer-unavailable = 3D vaizdavimo variklis nepasiekiamas (reikalinga wgpu sąsaja)
view-group = Grupė
# $path is the chain of group names, e.g. "Table › Leg".
view-editing = Redaguojama: Modelis › { $path }   (Esc – uždaryti)
view-hud-show-tip = Rodyti: ašis, žemę, dangų, tinklelį šiame rodinyje; pagalbines linijas, šešėlius, pjūvio plokštumas / pjūvius / užpildą visuose rodiniuose (taip pat meniu Rodinys)

## ======== Įrankių juosta ========

toolbar-size-small = Maža
toolbar-size-medium = Vidutinė
toolbar-size-large = Didelė

## Reserve headings

toolbar-group-draw = Braižymas
toolbar-group-tools = Įrankiai
toolbar-group-camera = Kamera
toolbar-group-file = Failas
toolbar-group-edit = Redaguoti
toolbar-group-snapping = Prisitraukimas
toolbar-group-window = Langas

## Buttons

toolbar-snapping-menu = Prisitraukimo meniu
toolbar-separator = Skirtukas
toolbar-snapping = Prisitraukimas
# $state is "off", "some" or "on".
toolbar-snapping-tip = Pasirinkite, prie kokių taškų ir krypčių prisitraukia žymeklis. { $state ->
        [off] Prisitraukimas išjungtas.
        [some] Kai kurie prisitraukimai išjungti.
       *[on] Visi prisitraukimai įjungti.
    }

## ======== Būsenos juosta ========

status-grid-step = Tinklelio žingsnis
status-measurements-tip = Matmenys: tiesiog įveskite reikšmę ir paspauskite Enter (pvz., 2500, 2.5m, 300,200, 24s)

## ======== Darbo plokštumos ========

workplane-title = Darbo plokštumos
workplane-none = Darbo plokštumų nėra. Pridėkite naudodami Įrankiai → Pagalbinė plokštuma.
workplane-active-tip = Aktyvi: braižoma šioje plokštumoje
workplane-visible = Matoma
workplane-deactivate = Išjungti
workplane-no-active-hint = Be aktyvios plokštumos naudojamos braižymo ašys.
# $name is the plane's name.
workplane-world = { $name } (pasaulio)
workplane-origin = Pradžios taškas
workplane-angles = Posvyris / azimutas / sukimas
workplane-tilt-tip = Posvyris: normalės kampas nuo mėlynos ašies
workplane-azimuth-tip = Azimutas: posvyrio kryptis nuo raudonos ašies
workplane-spin-tip = Sukimas aplink normalę
workplane-undo-delete = Naikinti darbo plokštumą
workplane-undo-edit = Redaguoti darbo plokštumas

## ======== Lapai ir vektorinis eksportas ========

layout-credits-none = Autorystė: atsisiųstų modelių nėra
# $lines is one line per downloaded model.
layout-credits = Autorystė:
    { $lines }
layout-col-name = Pavadinimas
layout-col-qty = Kiekis
layout-col-size = Dydis
layout-col-material = Medžiaga
layout-no-parts = Grupių ar komponentų nėra
layout-scale-as-shown = Kaip rodoma
# $n is the scale denominator.
layout-scale-label = Mastelis 1:{ $n }
layout-tb-author = Autorius
layout-tb-date = Data
layout-tb-scale = Mastelis
layout-tb-sheet = Lapas
# $n is the sheet number, $total the count of sheets.
layout-tb-sheet-of = { $n } iš { $total }

## ======== Nustatymai ========

prefs-language = Kalba
# { $language } is the name of the system's language.
prefs-language-system = Kaip sistemoje ({ $language })
prefs-language-note = Savo vertimai: įdėkite <code>.ftl į { DATA_DIR() }/locales/ (žr. docs/LOCALIZATION.md); jis taip pat pakeičia atskirus integruotos kalbos tekstus.

prefs-section-general = Bendra
prefs-section-drawing = Braižymas
prefs-section-libraries = Bibliotekos
prefs-section-downloads = Atsisiuntimai
prefs-section-view = Rodinys
prefs-section-navigation = Naršymas
prefs-section-interface = Sąsaja
prefs-section-shortcuts = Spartieji klavišai
prefs-section-toolbar = Įrankių juosta
prefs-section-render = Vaizdavimas

## Search words of the sections (besides their names; a translation lists its own)

prefs-words-general = automatinis išsaugojimas minutės šablonas naujas dokumentas vienetai atšaukimo žingsniai istorija atsarginė kopija ankstesnė versija tiltas python mcp di agentas scenarijai
prefs-words-render = vaizdavimas paveikslėlis dydis mėginiai atšokimai kokybė triukšmo šalinimas programos gif animacija oidn open image denoise
prefs-words-drawing = prisitraukimas nuspėjimas galinis taškas vidurio taškas briauna paviršius ašis tinklelis žingsnis pagrindinis žingsniai perkelti sukti mastelis kampas
prefs-words-libraries = komponentai medžiagos aplankas pavyzdžiai modelyje plytelės mažos didelės sąrašas
prefs-words-downloads = stebėti atsisiuntimai aplankas naršyklė
prefs-words-interface = tema šviesi tamsi sistema išvaizda mastelis dydis artinimas įrankių juosta piktogramos kalba vertimas pradžia paleidimas pradžios ekranas
prefs-words-navigation = pelė jutiklinis kilimėlis sukimas artinimas slinkimas greitis apversti ratukas žymeklis vidurinis mygtukas du pirštai gestas suspaudimas pasukimas pakreipimas daugelio lietimų
prefs-words-shortcuts = klaviatūra spartieji klavišai karštieji klavišai klavišai
prefs-words-toolbar = įrankių juosta mygtukai tvarka atsarga skirtukas pritaikyti įrankiai
prefs-words-view = naujas dokumentas stilius šešėliuotas tekstūros vienspalvis paslėptos linijos karkasas realistinis briaunos kontūrai ašys žemė dangus tinklelis šešėliai perspektyva lygiagreti projekcija fov matymo laukas kamera

## Window

prefs-title = Nustatymai
prefs-search = Paieška
prefs-nothing-found = Nieko nerasta
prefs-reset = Atkurti
prefs-reset-tip = Šio skyriaus nustatymai grąžinami į numatytuosius
# $section is the section's name.
prefs-defaults-flash = Nustatymai → { $section }: numatytieji

## Rows with a folder or a program

# $path is the folder.
prefs-folder-default = { $path } (numatytasis)
prefs-file-found = { $path } (rastas)
prefs-choose = Pasirinkti…
prefs-default = Numatytasis
prefs-show-in-finder = Rodyti failų tvarkyklėje
    .macos = Rodyti programoje Finder
    .windows = Rodyti failų naršyklėje

## General

prefs-data-folder = Duomenų aplankas
prefs-data-folder-note = Čia laikomi papildiniai, bibliotekos, šablonai, jūsų vertimai ir tiltas Python scenarijams bei DI agentams (bridge.json, python/).
prefs-autosave = Automatinis išsaugojimas
prefs-autosave-every = Automatiškai išsaugoti kas
prefs-minutes = min.
prefs-autosave-note = Neišsaugoti pakeitimai įrašomi į { DATA_DIR() }/autosave, o ne šalia failo.
prefs-new-document = Naujas dokumentas
prefs-template-note = Komandos Failas → Naujas šablonas (vienetai ir mastelis); jo rodinys nustatomas skyriuje Rodinys.
prefs-undo = Atšaukti
prefs-keep = Saugoti
prefs-undo-steps = atšaukimo žingsnių kiekvienam dokumentui
prefs-undo-note = Kiekvienas žingsnis saugo modelio kopiją: dideli modeliai su daug žingsnių užima daugiau atminties.
prefs-appimage-menu = Programų meniu (AppImage)
prefs-appimage-add = Pridėti į programų meniu
prefs-appimage-remove = Pašalinti iš programų meniu
prefs-appimage-note = Šio AppImage meniu įrašas, piktograma ir .vrst failų tipas, tik jums (~/.local/share). Perkėlę AppImage failą, paleiskite jį vieną kartą — įrašas atsinaujins.
appimage-offer-title = Pridėti Varstotas į programų meniu?
appimage-offer-text = Šio AppImage nėra programų meniu, todėl doke rodoma standartinė piktograma, o .vrst failai neatsidaro su Varstotas. Pridėti meniu įrašą, piktogramą ir .vrst failų tipą jums (~/.local/share)? Pašalinti juos galima Preferences → General.
appimage-offer-add = Pridėti į meniu
appimage-offer-no = Ne, ačiū
appimage-added = Varstotas pridėtas į programų meniu
appimage-removed = Varstotas pašalintas iš programų meniu
appimage-failed = Nepavyko pakeisti programų meniu įrašo: { $error }
prefs-bridge = Scenarijai ir DI agentai
prefs-bridge-external = Leisti jungtis scenarijams ir DI agentams už Varstotas ribų (MCP)
prefs-bridge-note = Išjungta: tiltą pasiekia tik iš Varstotas paleisti scenarijai (Failas → Vykdyti scenarijų…, papildiniai). Įjungta: { DATA_DIR() }/bridge.json saugo vietinį adresą ir raktą kitoms programoms, pvz., MCP serveriui DI asistentui. Tiltas klausosi tik 127.0.0.1.

## Render

prefs-new-documents = Nauji dokumentai
prefs-match-view = Pritaikyti prie rodinio
prefs-match-view-tip = Aukštis atitinka vaizdo srities formą
prefs-denoise = Šalinti triukšmą sustabdžius vaizdavimą
prefs-denoise-ai = Naudojant Open Image Denoise, jei įdiegta
prefs-use-current-doc = Naudoti dabartinio dokumento
prefs-use-current-doc-tip = Perimti atidaryto dokumento vaizdavimo nustatymus
prefs-apply-current-doc = Taikyti dabartiniam dokumentui
prefs-apply-current-doc-tip = Atidarytas dokumentas gauna šiuos nustatymus (Atšaukti juos grąžina)
prefs-render-note = Failas → Naujas pradedamas su šiais; Langas → Vaizdavimas keičia atidaryto dokumento nustatymus.
prefs-programs = Programos
prefs-oidn = Open Image Denoise biblioteka (vaizdavimo triukšmo šalinimas)
prefs-oidn-missing = nerasta: triukšmą šalina integruotas filtras (įdiekite Open Image Denoise: brew install open-image-denoise arba savo sistemos paketą)

## Drawing

prefs-grid = Tinklelis
prefs-show-grid = Rodyti tinklelį
prefs-grid-step = žingsnis
prefs-grid-major = pagrindinė linija kas
prefs-grid-note = Tolinant žingsnis dauginamas iš pagrindinių linijų skaičiaus. Kai aktyvus įrankis „Pasirinkti“, įvedus ilgį nustatomas žingsnis.
prefs-snapping = Prisitraukimas ir žingsniai
prefs-snapping-note = Tie patys jungikliai kaip prisitraukimo meniu.

## Libraries

prefs-components = Komponentai
prefs-library-folder = Bibliotekos aplankas
prefs-components-folder-tip = Kortelės Komponentai komponentai (.vrst) ir generatoriai (.rhai)
prefs-show-examples = Rodyti pavyzdžius
prefs-show-examples-tip = Kartu pateikiami pavyzdiniai generatoriai
prefs-show-in-model = Rodyti modelyje
prefs-show-in-model-tip = Atidaryto dokumento komponentai ir generatoriai
prefs-materials = Medžiagos
prefs-materials-folder-tip = Kortelės Medžiagos medžiagos (.vrstmat); poaplankiai yra kategorijos
prefs-show-mat-examples-tip = Kartu su Varstotas pateikiamos medžiagos (demonstracija „Medžiagų galerija“)
prefs-tiles = Plytelės
prefs-tiles-note = Abi kortelės; taip pat jų … meniu.

## Downloads

prefs-watch-downloads = Stebėti atsisiuntimus
prefs-watch-offer = Siūlyti importuoti naujus modelių failus
prefs-watch-offer-tip = Kiekvienas naujas modelio failas ar archyvas aplanke siūlomas importuoti (visada įjungta, kol laukiama puslapio iš Kitų svetainių)
prefs-watched-folder = Stebimas aplankas
prefs-watched-folder-tip = Kur naršyklė įrašo failus
prefs-downloads-note = Taip pat Langas → Internetinė 3D biblioteka → Kitos svetainės.

## Toolbar

prefs-toolbar-note = Įrankių juostos mygtukai pagal eilę; kiti laukia atsargoje (jie lieka meniu).
prefs-toolbar-on = Įrankių juostoje
prefs-toolbar-reserve = Atsarga
prefs-toolbar-to-reserve = Į atsargą
prefs-toolbar-down = Žemyn
prefs-toolbar-up = Aukštyn
prefs-toolbar-to-end = Į įrankių juostos pabaigą

## Interface

prefs-theme = Tema
prefs-theme-light = Šviesi
prefs-theme-dark = Tamsi
prefs-theme-system = Kaip sistemoje
prefs-theme-note = Vaizdo sritis išlaiko savo spalvas (dangus, žemė); lapai ir išklotinės lieka baltas popierius.
prefs-scale = Mastelis
prefs-scale-note = Meniu, skydeliai ir langai; %CMDK%+ / %CMDK%− / %CMDK%0 taip pat jį keičia.
prefs-toolbar-icons = Įrankių juostos piktogramos

## Shortcuts

prefs-shortcuts-note = Spustelėkite spartųjį klavišą, tada paspauskite naujus klavišus: Esc atšaukia, Backspace pašalina spartųjį klavišą. Klavišas, kurį turėjo kita komanda, iš jos atimamas.
# $command is the command's name.
prefs-shortcut-removed = { $command }: be sparčiojo klavišo
# $keys is the pressed shortcut, $why the reason.
prefs-shortcut-reserved = { $keys } negali būti spartusis klavišas: { $why }
# $other is the command that had the shortcut before.
prefs-shortcut-taken = { $command }: { $keys } (atimta iš: { $other })
prefs-shortcut-set = { $command }: { $keys }
prefs-shortcuts-tools = Įrankiai
prefs-press-keys = Paspauskite klavišus…
prefs-default-none = Numatytasis: nėra
# $keys is the list of default shortcuts.
prefs-default-keys = Numatytasis: { $keys }

## Navigation

prefs-speed = Greitis
prefs-nav-orbit = Sukimas
prefs-nav-pan = Slinkimas
prefs-nav-zoom = Artinimas
prefs-invert-wheel = Apversti ratuką
prefs-invert-wheel-tip = Ratukas nuo savęs tolina
prefs-zoom-cursor = Artinti link žymeklio
prefs-zoom-cursor-tip = Išjungta: link rodinio vidurio
prefs-mouse = Pelė
prefs-middle-drag = Vidurinio mygtuko tempimas:
prefs-shift-other = Shift suteikia kitą.
prefs-trackpad = Jutiklinis kilimėlis
prefs-two-fingers = Du pirštai:
prefs-trackpad-note = Shift sukeičia sukimą ir stumdymą. Suspaudimas ir %CMD% + du pirštai artina nepriklausomai nuo pelės ratuko.

## View (defaults of a new document)

prefs-view-note = Kaip atrodo naujas dokumentas (Failas → Naujas, Naujas iš šablono). Meniu Rodinys ir vaizdo srities mygtukai keičia dabartinį rodinį.
prefs-camera = Kamera
prefs-fov = Matymo laukas
prefs-use-current-view = Naudoti dabartinį rodinį
prefs-use-current-view-tip = Perimti aktyvios vaizdo srities stilių, jungiklius ir kamerą
prefs-apply-current-view = Taikyti dabartiniam rodiniui
prefs-apply-current-view-tip = Nustatyti aktyvią vaizdo sritį kaip naują dokumentą

## ======== Spartieji klavišai ========

key-new = Naujas
key-open = Atidaryti…
key-save = Išsaugoti
key-save-as = Išsaugoti kaip…
key-quit = Išeiti
key-undo = Atšaukti
key-redo = Grąžinti
key-select-all = Pasirinkti viską
key-select-none = Nieko nepasirinkti
key-make-group = Sukurti grupę
key-zoom-extents = Rodyti viską
key-toggle-snapping = Prisitraukimas įj. / išj.
key-next-scene = Kita scena
key-prev-scene = Ankstesnė scena

## Key names and reserved keys (Preferences → Shortcuts)

key-name-space = Tarpas
key-name-pgdn = PgDn
key-name-pgup = PgUp
key-reserved-tools = šį klavišą naudoja visi įrankiai
key-reserved-arrows = rodyklės užrakina nuspėjimo ašis
key-reserved-f1 = F1 atveria vadovą
key-reserved-digits = skaitmenys įvedami į Matmenis
key-reserved-measurements = šis klavišas įveda į Matmenis
key-reserved-clipboard = Iškirpti / Kopijuoti / Įklijuoti

## ======== Šablonai ========

template-architectural-mm = Architektūrinis — milimetrai
template-architectural-m = Architektūrinis — metrai
template-woodworking-mm = Medžio apdirbimas — milimetrai
template-imperial = Anglosaksiškas — coliai

template-architectural-mm-desc = Pastatai ir interjerai, ilgiai milimetrais.
template-architectural-m-desc = Pastatai ir sklypai, ilgiai metrais.
template-woodworking-mm-desc = Baldai ir smulkūs objektai: artimas rodinys, ilgiai milimetrais.
template-imperial-desc = Ilgiai coliais.

## ======== Šoninė juosta ========

## Cards (titles are ids in the preferences; these are the shown names)

tray-card-entity-info = Savybės
tray-card-materials = Medžiagos
tray-card-material-editor = Medžiagos redaktorius
tray-card-components = Komponentai
tray-card-tags = Žymės
tray-card-outliner = Struktūra
tray-card-scenes = Scenos
tray-card-shadows = Šešėliai
tray-card-environment = Aplinka
tray-card-soften-edges = Glotninti briaunas
tray-card-instructor = Patarimai
tray-card-model-info-tip = Dokumento informacija: dokumento statistika, pašalinti nenaudojamus
tray-cards-all-off = Visos kortelės išjungtos: Langas → Šoninė juosta

tray-untagged = Be žymės
tray-default = Numatytoji
tray-rename = Pervadinti
tray-invalid-value = Netinkama reikšmė
# $error is the reason.
tray-kernel-not-ready = (branduolys nepasiruošęs: { $error })
tray-through = Per

## Shadows

tray-shadows-show = Rodyti šešėlius
tray-shadows-time = Laikas
tray-shadows-date = Data
tray-shadows-light = Šviesa
tray-shadows-dark = Tamsa
tray-shadows-display = Rodymas
tray-shadows-on-faces = Ant paviršių
tray-shadows-on-ground = Ant žemės
tray-shadows-location = Vietovė
tray-shadows-latitude = Platuma
tray-shadows-longitude = Ilguma
tray-shadows-utc-offset = UTC poslinkis
tray-shadows-north-angle = Šiaurės kampas
# $alt is the sun's altitude in degrees.
tray-sun-above = Saulė { $alt }° virš horizonto
tray-sun-below = Saulė žemiau horizonto

## Soften Edges

tray-soften-allow-normals = Leisti normales tarp
tray-soften-smooth-normals = Glotnios normalės
tray-soften-coplanar = Glotninti vienoje plokštumoje esančius
tray-soften-select = Pasirinkite briaunas ar paviršius
# $n is the number of edges.
tray-soften-edges-count = Briaunų: { $n }

## Savybės

tray-info-invalid-context = Netinkamas kontekstas
tray-info-no-selection = Nieko nepasirinkta
tray-info-units = Vienetai
tray-unit-millimeters = Milimetrai
tray-unit-centimeters = Centimetrai
tray-unit-meters = Metrai
tray-unit-inches = Coliai
tray-unit-feet = Pėdos
tray-info-faces = Paviršiai
tray-info-edges = Briaunos
tray-info-groups = Grupės
tray-info-components = Komponentai
tray-info-face = Paviršius
tray-info-area = Plotas
tray-info-front = Priekis
tray-info-back = Galas
tray-info-tag = Žymė
tray-info-edge = Briauna
tray-info-length = Ilgis
tray-info-soft = Minkšta
tray-info-smooth = Glotni
tray-info-group = Grupė
tray-info-component = Komponentas
tray-info-parametric-group = Parametrinė grupė
# $kind is Group, Component or Parametric Group.
tray-info-solid = Vientisas: { $kind }
tray-info-volume = Tūris
tray-info-instance = Egzempliorius
tray-info-name-hint = pavadinimas
tray-info-definition = Apibrėžtis
tray-info-material = Medžiaga
tray-info-source = Šaltinis
tray-info-licence-unknown = licencija nežinoma
# $line is the credit line of the model.
tray-info-source-hover =
    { $line }
    Failas → Eksportuoti → Autorystė išvardija modelio autorius
# $count is the number of selected items.
tray-info-n-guides = Pagalbinių linijų: { $count }
tray-info-n-dimensions = Matmenų: { $count }
tray-info-n-entities = Objektų: { $count }
tray-info-contains = Turi
tray-count-faces = { $n ->
        [one] { $n } paviršius
        [few] { $n } paviršiai
       *[other] { $n } paviršių
    }
tray-count-edges = { $n ->
        [one] { $n } briauna
        [few] { $n } briaunos
       *[other] { $n } briaunų
    }
tray-count-groups = { $n ->
        [one] { $n } grupė
        [few] { $n } grupės
       *[other] { $n } grupių
    }
tray-count-components = { $n ->
        [one] { $n } komponentas
        [few] { $n } komponentai
       *[other] { $n } komponentų
    }
tray-count-guides = { $n ->
        [one] { $n } pagalbinė linija
        [few] { $n } pagalbinės linijos
       *[other] { $n } pagalbinių linijų
    }
tray-count-dimensions = { $n ->
        [one] { $n } matmuo
        [few] { $n } matmenys
       *[other] { $n } matmenų
    }

## Undo step names

tray-undo-rename = Pervadinti
tray-undo-soften-edge = Glotninti briauną
tray-undo-unsoften-edge = Nebeglotninti briaunos
tray-undo-set-tag = Nustatyti žymę
tray-undo-texture-position = Tekstūros padėtis
tray-undo-edit-material = Redaguoti medžiagą
tray-undo-edit-segments = Redaguoti segmentus
tray-undo-edit-radius = Redaguoti spindulį
tray-undo-edit-text = Redaguoti tekstą
tray-undo-edit-dimension-text = Redaguoti matmens tekstą
tray-undo-rename-section = Pervadinti pjūvį
tray-undo-delete-tag = Naikinti žymę
tray-undo-rename-tag = Pervadinti žymę
tray-undo-add-tag = Pridėti žymę
tray-undo-scene-properties = Scenos savybės
tray-undo-rename-scene = Pervadinti sceną

## Parametric group

tray-params-make = Padaryti parametrine
tray-params-make-hover = Dydis ir kopijos nustatomi Savybėse
tray-params-title = Parametrai
tray-params-length = Ilgis (X)
tray-params-width = Plotis (Y)
tray-params-height = Aukštis (Z)
tray-params-gap = Tarpas
tray-params-copies = Kopijos
tray-params-along = Išilgai
tray-params-resize = Keisti dydį
tray-params-stretch = Tempti
tray-params-stretch-hover = Perkelti tolimesnę pusę: galai išlaiko dydį
tray-params-scale = Mastelis

## Texture of a face

tray-tex-title = Tekstūra
tray-tex-offset-x = Poslinkis X
tray-tex-offset-y = Poslinkis Y
tray-tex-rotation = Pasukimas
tray-tex-scale-x = Mastelis X
tray-tex-scale-y = Mastelis Y
tray-tex-reset = Atkurti
tray-tex-tile-w = Plytelės plotis
tray-tex-tile-h = Plytelės aukštis
uv-title = UV redaktorius
uv-undo = UV keitimas
uv-undo-mark-seams = Pažymėti siūles
uv-undo-clear-seams = Pašalinti siūles
uv-seams-marked = Kraštinių pažymėta siūlėmis: { $count }.
uv-seams-cleared = Pašalinta siūlių: { $count }.
uv-no-edges = Pirmiausia pažymėkite kraštines (kartu su paviršiais).
uv-nothing = Nėra ką keisti.
uv-empty = Pažymėkite paviršius su tekstūra (arba nudažykite juos), kad keistumėte tekstūros išdėstymą.
uv-unfold-hover = Kiekvieną salą paguldyti plokščiai tikru dydžiu, perkirpus per siūles
uv-planar = Plokščia
uv-box = Kubinė
uv-cylinder = Cilindrinė
uv-sphere = Sferinė
uv-seams-mark-hover = Pažymėti pasirinktas kraštines kaip siūles
uv-seams-clear-hover = Pašalinti pasirinktų kraštinių siūles (kitaip visas)
uv-seams-hard-hover = Pažymėti siūle kiekvieną kietą (nesušvelnintą) kraštinę tarp paviršių
uv-rotate-ccw = Pasukti 90° prieš laikrodžio rodyklę
uv-rotate = Pasukti 90° pagal laikrodžio rodyklę
uv-flip-u = Apversti kairė ↔ dešinė
uv-flip-v = Apversti viršus ↕ apačia
uv-fit-hover = Kiekvieną salą išdidinti per vieną plytelę
uv-pack-hover = Išdėstyti salas vienoje plytelėje vienu masteliu
uv-align-hover = Pasukti kiekvieną salą, kad ilgiausia jos kraštinė eitų tiesiai
uv-reset-hover = Atstatyti tekstūros padėtį salose (grąžinti pradinį uždėjimą)
uv-tool-select = Žymėjimas: spustelėkite salą (veidų režimu – veidą), Shift+spustelėjimas prideda arba atima, rėmelis – tempkite tuščioje vietoje; pažymėtą tempiant ji perkeliama
uv-tool-move = Perkelti pažymėtas salas ar veidus (raudona / žalia gizmo rodyklė – viena ašimi)
uv-tool-rotate = Sukti pažymėtas salas ar veidus apie atramos tašką
uv-tool-scale = Keisti pažymėtų salų ar veidų mastelį nuo atramos taško (raudonas / žalias gizmo kvadratas – tik viena ašimi)
uv-mode-island = Žymėti salas ištisai
uv-mode-face = Žymėti atskirus veidus
uv-select-all = Pažymėti viską
uv-frame = Rodyti visas salas
uv-project-hover = Tekstūros projekcija: plokščia, dėžė, cilindras ar sfera
uv-reset-rotation = Atstatyti pasukimą: atsukti salas, kad tekstūra eitų tiesiai (išklotinės forma išlieka)
uv-reset-scale = Atstatyti mastelį: grąžinti tekstūrai jos dydį (išklotinės forma išlieka)
uv-hint-select = Spustelėjimas / rėmelis: žymėti · spustelėjus dar kartą: esanti žemiau · Shift: pridėti ar atimti · tempti pažymėtą: perkelti · ratukas: mastelis · dešinysis: slinkti
uv-hint-move = Tempti: perkelti pažymėtą · gizmo rodyklės: tik viena ašimi · limpa prie kampų ir plytelių tinklelio (Alt: be lipimo; Shift: žingsniai įj./išj.) · įveskite dx; dy
uv-hint-rotate = Tempti: sukti apie atramos tašką · gizmo centrą galima tempti, dukart spustelėjus – į centrą · Shift: kampo žingsnis įj./išj. · įveskite kampą
uv-hint-scale = Tempti: mastelis nuo atramos taško · raudonas / žalias kvadratas: viena ašis, oranžinis: abi · gizmo centrą galima tempti, dukart spustelėjus – į centrą · Shift: žingsniai įj./išj. · įveskite k arba kx; ky
uv-value-move = Poslinkis
uv-value-rotate = Kampas
uv-value-scale = Mastelis
uv-value-hover = Tiksli reikšmė, Enter pritaiko; iškart po tempimo ją pakeičia
uv-checker-hover = Rodyti šachmatų lentą vietoje tekstūrų (čia ir 3D vaizde), kad matytųsi ištempimai
uv-density = Tekselių tankis
uv-px-per-m = px/m
uv-summary = Paviršių: { $faces } · salų: { $islands } · pasirinkta: { $selected }
uv-material = Medžiaga: { $name }
uv-map-hover = Kuris medžiagos žemėlapis rodomas po išklotine
uv-no-texture = Nėra tekstūros: paviršiai ir dabartinė medžiaga neturi paveikslėlio
uv-tile-hover = Kartoti tekstūrą per visą lauką (kitaip – viena plytelė)
tray-tex-width = Tekstūros plotis
tray-tex-height = Tekstūros aukštis
tray-tex-keep-aspect = Išlaikyti proporcijas

## Curves

tray-curve = Kreivė
tray-curve-closed = Uždara kreivė
tray-curve-points = Taškai
# $count is the number of points, $smooth how many of them are smooth.
tray-curve-points-value = { $count } (glotnių: { $smooth })
tray-curve-segments = Segmentai
tray-curve-segments-hover = Briaunų skaičius tarp dviejų taškų esančiame lenktame ruože
tray-curve-edit = Redaguoti kreivę
tray-curve-edit-hover = Tempkite jos taškus ir rankenėles (arba dukart spustelėkite kreivę)
tray-curve-arc = Lankas
tray-curve-circle = Apskritimas
tray-curve-polygon = Daugiakampis
tray-curve-radius = Spindulys
tray-curve-sides = Kraštinės
tray-curve-circumference = Ilgis
tray-curve-arc-length = Lanko ilgis
tray-curve-angle = Kampas

## Dimension, light, section plane, guides

tray-dim-text = Tekstas
tray-dim-linear = Linijinis matmuo
tray-light-title = Šviesa
tray-light-render = Vaizdavimas…
tray-light-render-hover = Langas → Vaizdavimas
tray-light-hint = Perkelti / Sukti ją perkelia ir pasuka; tempkite kvadratinę rankenėlę, kad nusitaikytumėte.
tray-section-title = Pjūvio plokštuma
tray-section-name = Pavadinimas
tray-section-active-cut = Aktyvus pjūvis
tray-section-normal = Normalė
tray-section-reverse = Apversti
tray-section-align-view = Lygiuoti rodinį
tray-guide-line = Pagalbinė linija
tray-guide-direction = Kryptis
tray-guide-point = Pagalbinis taškas
tray-guide-position = Padėtis

## Materials card

tray-mat-current = Dabartinė dažymo medžiaga (B)
tray-mat-custom = Pasirinktinė
tray-mat-use = Naudoti

## Tags

tray-tags-add = Pridėti žymę
tray-tags-header = Matoma · Pavadinimas
tray-tags-untagged-hover = Perjungti viršutinio lygio geometrijos, grupių ir komponentų be žymės matomumą (jų turinys priklauso jiems)
tray-tags-toggle = Perjungti matomumą
tray-tags-rename-hover = Dukart spustelėkite, kad pervadintumėte; spustelėkite dešiniuoju mygtuku, kad pamatytumėte daugiau
tray-tags-delete-hover = Naikinti žymę (tai, kas ja pažymėta, taps be žymės)
# $n is the number of the new tag.
tray-tag-new-name = Žymė { $n }
# $name is the tag, $uses how many items were tagged with it.
tray-tag-deleted = Ištrinta žymė „{ $name }“
tray-tag-deleted-uses = Ištrinta žymė „{ $name }“: elementų, kurie dabar be žymės: { $uses }

## Outliner

tray-outliner-empty = Grupių ar komponentų dar nėra.
tray-outliner-hide-show = Slėpti / rodyti
tray-outliner-zoom = Priartinti prie
tray-outliner-zoom-hover = Pasirinkti ir parodyti rodinyje (dukart spustelėjus)
tray-outliner-edit-group = Redaguoti grupę
tray-outliner-edit-component = Redaguoti komponentą
tray-outliner-unhide = Rodyti
tray-outliner-hide = Slėpti
tray-outliner-explode = Išskaidyti
tray-outliner-make-unique = Padaryti unikalų
tray-outliner-make-unique-hover = Sava apibrėžtis: redaguojant jos kitos kopijos nebekeičiamos
tray-outliner-move-out = Iškelti iš grupės
tray-outliner-move-out-hover = Į aplink šią esančią grupę / komponentą (arba tempkite eilutę)

## Scenes

tray-scenes-add = Pridėti sceną iš dabartinio rodinio
tray-scenes-update = Atnaujinti pasirinktą sceną pagal dabartinį rodinį
tray-scenes-delete = Naikinti pasirinktą sceną
tray-scenes-up = Perkelti sceną aukštyn
tray-scenes-down = Perkelti sceną žemyn
tray-scenes-play = Leisti
tray-scenes-pause = Pristabdyti
tray-scenes-play-hover = Leisti scenų animaciją (Rodinys → Animacija)
tray-scenes-empty = Scenų nėra. Spustelėkite +, kad išsaugotumėte dabartinį rodinį.
tray-scenes-click-hover = Spustelėkite, kad pereitumėte į šią sceną, dukart spustelėkite, kad pervadintumėte
tray-scenes-include = Įtraukti į animaciją
tray-scenes-animation = Animacija
tray-scenes-transitions = Scenų perėjimai
tray-scenes-transition = Perėjimas
tray-scenes-delay = Scenos trukmė

## Patarimai

# $key is the shortcut.
tray-instructor-shortcut = Spartusis klavišas: { $key }
tray-instructor-more = Daugiau Vartotojo vadove (F1)
tray-instructor-not-implemented = Šis įrankis dar neįgyvendintas.

## Library cards, shared frame (tray_lib)

tray-tiles-small = Mažos plytelės
tray-tiles-large = Didelės plytelės
tray-tiles-list = Sąrašas
tray-lib-search = Paieška
tray-lib-refresh = Atnaujinti: iš naujo perskaityti bibliotekos aplanką
tray-lib-more = Bibliotekos aplankas, ką rodyti, plytelių dydis
tray-lib-section-show = Rodyti skyrių
tray-lib-section-fold = Suskleisti skyrių
# $unused is the number of unused items.
tray-purge-materials = Pašalinti nenaudojamus: pašalinti { $unused } medžiagų, kurių modelyje niekas nenaudoja
tray-purge-materials-none = Pašalinti nenaudojamus: nenaudojamų medžiagų nėra
tray-purge-components = Pašalinti nenaudojamus: pašalinti { $unused } komponentų, kurių modelyje niekas nenaudoja
tray-purge-components-none = Pašalinti nenaudojamus: nenaudojamų komponentų nėra
tray-lib-fav-add = Pridėti prie mėgstamiausių
tray-lib-fav-remove = Pašalinti iš mėgstamiausių
tray-lib-move-to-folder = Perkelti į aplanką
tray-my-library = Mano biblioteka
# $path is the file that is in the way.
tray-lib-exists = { $path } jau yra
tray-folder-new = Naujas aplankas…
tray-folder-rename = Pervadinti aplanką…
tray-folder-delete = Naikinti aplanką…
tray-folder-delete-files = Naikinti aplanką ir jo failus…

## ======== Komponentų biblioteka ========

## Sections

library-examples = Pavyzdžiai
library-examples-hover = Kartu su Varstotas pateikiami generatoriai (ne failai)
library-in-model = Modelyje
library-in-model-hover = Atidaryto dokumento komponentai ir generatoriai (Išsaugoti bibliotekoje sukuria failą)
library-favourites = Mėgstamiausi
library-favourites-hover = Spustelėkite bibliotekos plytelę dešiniuoju mygtuku → Pridėti prie mėgstamiausių
library-recent = Naujausi
library-recent-hover = Paskutiniai įdėti bibliotekos komponentai
library-hide-examples = Slėpti kartu pateikiamus pavyzdžius (… → Rodyti pavyzdžius juos grąžina)
library-hide-in-model = Slėpti dokumento komponentus (… → Rodyti modelyje juos grąžina)
# $path is the folder.
library-folder-reveal-hover =
    { $path }
    Spustelėkite, kad parodytumėte programoje Finder
    .windows =
    { $path }
    Spustelėkite, kad parodytumėte failų naršyklėje
library-show-in-finder = Rodyti programoje Finder
    .windows = Rodyti failų naršyklėje
    .linux = Rodyti failų tvarkyklėje

## Top row and the … menu

library-plus-hover = Išsaugoti pasirinkimą bibliotekoje, naujas generatorius
library-online-hover = Internetinė 3D biblioteka: nemokami modeliai iš interneto (Poly Haven, Smithsonian, NASA, Kenney, papildiniai)
library-save-selection = Išsaugoti pasirinkimą bibliotekoje…
library-select-first = Pirmiausia ką nors pasirinkite
library-new-generator = Naujas generatorius…
library-new-generator-hover = Scenarijaus sukurtas komponentas su parametrais (Langas → Generatorių redaktorius)
library-choose-folder = Pasirinkti aplanką…
library-default-folder = Numatytasis aplankas
library-default-folder-hover = Atgal į { DATA_DIR() }/components
library-purge = Pašalinti nenaudojamus komponentus
library-purge-hover = Pašalinti komponentus (ir grupių apibrėžtis), kurių modelyje niekas nenaudoja
library-show-examples = Rodyti pavyzdžius
library-show-examples-hover = Kartu pateikiami pavyzdiniai generatoriai
library-show-in-model = Rodyti modelyje
library-show-in-model-hover = Atidaryto dokumento komponentai ir generatoriai
library-empty = Komponentų dar nėra: pasirinkite ką nors ir naudokite + → Išsaugoti pasirinkimą bibliotekoje.
library-nothing-found = Nieko nerasta
library-image-filter = Paveikslėlis
library-name = Pavadinimas
library-save = Išsaugoti
# $n is the number of items.
library-unreadable-files = Nepavyko perskaityti failų: { $n }
library-unreadable-generators = Nepavyko perskaityti generatorių: { $n }

## Tiles and their context menus

# $name is the component, $note an extra line (ends with a line break, or empty), $x $y $z the size.
library-tile-hover =
    { $name }
    { $note }{ $x } × { $y } × { $z }
    Spustelėkite, kad įdėtumėte
# $uses is how many copies are placed.
library-in-document = Šiame dokumente: įdėta { $uses }
library-place = Įdėti
library-select-instances = Pasirinkti egzempliorius
library-preview-from-view = Nustatyti peržiūrą iš rodinio
library-preview-from-view-file-hover = Dabartinis vaizdo srities vaizdas tampa paveikslėliu (įrašomas faile): pirmiausia sukadruokite modelį
library-preview-from-view-def-hover = Dabartinis vaizdo srities vaizdas tampa plytelės paveikslėliu (įrašomas su dokumentu): pirmiausia sukadruokite komponentą
library-preview-image = Nustatyti peržiūros paveikslėlį…
library-preview-image-file-hover = Paveikslėlis, rodomas čia vietoj nupiešto modelio (įrašomas faile)
library-preview-image-def-hover = Paveikslėlis, rodomas plytelėje vietoj sugeneruoto modelio
library-rename-ellipsis = Pervadinti…
library-delete-ellipsis = Naikinti…
library-save-to-library = Išsaugoti bibliotekoje
library-save-to-library-hover = Įrašyti į bibliotekos aplanką, kad būtų prieinama bet kuriam dokumentui
library-parameters = Parametrai…
library-parameters-hover = Langas → Komponento parametrai: reikšmės, kurios perkelia, rodo ar dažo jo dalis
library-delete-from-document = Naikinti iš dokumento…
library-delete-from-document-hover = Komponentas ir visos jo įdėtos kopijos

## Preview pictures

library-undo-set-preview = Nustatyti peržiūros paveikslėlį
library-undo-remove-preview = Pašalinti peržiūros paveikslėlį
library-undo-set-def-preview = Nustatyti komponento peržiūrą
library-undo-remove-def-preview = Pašalinti komponento peržiūrą
library-preview-set-file = Peržiūros paveikslėlis nustatytas (įrašytas faile)
library-preview-removed = Peržiūros paveikslėlis pašalintas
# $name is the component.
library-preview-set-def = Komponento { $name } peržiūros paveikslėlis nustatytas (įrašytas su dokumentu)
library-preview-removed-def = Komponento { $name } peržiūros paveikslėlis pašalintas
# $path is the image file, $error the reason.
library-preview-error-file = Peržiūros paveikslėlis: { $path }: { $error }
library-preview-error = Peržiūros paveikslėlis: { $error }

## Saving, placing, moving

# $path is a file, $name a component, $error the reason.
library-exists = { $path } jau yra
library-save-error = Išsaugoti bibliotekoje: { $error }
library-saved = Išsaugota bibliotekoje: { $path }
library-save-failed = Nepavyko išsaugoti bibliotekoje: { $error }
library-place-failed = Nepavyko įdėti { $name }: { $error }
library-no-copies = Šio komponento kopijų viršutiniame modelio lygyje nėra
library-move-failed = Perkelti į aplanką: { $error }

## Folder dialogs

library-folder-new-title = Naujas aplankas
library-folder-rename-title = Pervadinti aplanką
library-folder-name = Aplanko pavadinimas
library-create = Sukurti
library-rename = Pervadinti
# $heading is the dialog's title, $name the folder, $error the reason.
library-folder-done = { $heading }: { $name }
library-folder-failed = { $heading }: { $error }
library-folder-delete-title = Naikinti aplanką?
# $name is the folder.
library-folder-delete-text-materials = Aplankas „{ $name }“ bus ištrintas iš medžiagų bibliotekos:
library-folder-delete-text-components = Aplankas „{ $name }“ bus ištrintas iš komponentų bibliotekos:
library-folder-empty = Aplankas tuščias.
# $files is a number of files, $dirs a number of subfolders.
library-folder-what = failų: { $files }
library-folder-what-dirs = failų: { $files }, poaplankių: { $dirs }
# $what is the library-folder-what text.
library-folder-not-empty = ⚠ Aplankas nėra tuščias: kartu bus ištrinta ({ $what }).
# $n is how many files are not listed.
library-folder-more = … ir dar { $n }
library-undone-copies-stay = Šio veiksmo atšaukti negalima. Dokumentuose jau įdėtos kopijos lieka.
library-folder-delete-with = Naikinti kartu su failais: { $files }
library-folder-deleted = Ištrintas aplankas { $name }
library-folder-deleted-files = Ištrintas aplankas { $name } ir failai: { $files }
library-folder-delete-failed = Naikinti aplanką: { $error }

## Delete dialogs

library-delete-title = Naikinti iš bibliotekos?
library-delete-text = „{ $name }“ bus ištrintas iš bibliotekos aplanko:
library-delete-doc-title = Naikinti iš dokumento?
library-delete-gen-text = Generatorius „{ $name }“ bus pašalintas iš šio dokumento.
# $uses is the number of placed components.
library-delete-gen-uses = Įdėti komponentai ({ $uses }) lieka paprastais komponentais, be parametrų.
library-undo-brings-back = Redaguoti → Atšaukti jį sugrąžina.
library-undo-brings-them-back = Redaguoti → Atšaukti juos sugrąžina.
library-undo-delete-generator = Naikinti generatorių { $name }
# $n is the number of components left without parameters.
library-generator-removed-uses = Pašalintas generatorius { $name }: komponentai be parametrų: { $n }
library-generator-removed = Pašalintas generatorius { $name }
library-deleted = Ištrinta iš bibliotekos: { $name }
library-delete-failed = Naikinti { $path }: { $error }
library-undo-delete-component = Naikinti komponentą { $name }
# $n is the number of removed copies.
library-component-deleted-copies = Ištrintas { $name } ir įdėtos kopijos: { $n }
library-component-deleted = Ištrintas { $name } iš dokumento
library-delete-def-text = „{ $name }“ įdėtas kartų: { $uses }. Komponentas ir visos jo kopijos bus pašalinti.

## Rename dialog

library-rename-material = Pervadinti medžiagą
library-rename-component = Pervadinti komponentą
# $file is the library file.
library-rename-file-too = Bibliotekos failas { $file } taip pat pervadinamas.
library-undo-rename-component = Pervadinti komponentą { $name }
library-renamed = Pervadinta į { $name }
library-rename-failed = Pervadinti { $path }: { $error }

## ======== Medžiagos ========

## Library card

mat-plus-hover = Nauja medžiaga iš paveikslėlio, importuoti medžiagų failus
mat-online-hover = Internetinė medžiagų biblioteka: nemokamos PBR medžiagos (Poly Haven, ambientCG, papildiniai)
mat-new-from-texture = Nauja iš tekstūros…
mat-new-from-texture-hover = Medžiaga iš paveikslėlio failo (PNG, JPEG)
mat-new-procedural = Nauja procedūrinė…
mat-new-procedural-hover = Vientisa tekstūra pagal šabloną: plytos, plytelės, lentos, triukšmas arba šachmatai
proc-title = Procedūrinė medžiaga
proc-bricks = Plytos
proc-tiles = Plytelės
proc-planks = Lentos
proc-noise = Triukšmas
proc-checker = Šachmatai
proc-main = Spalva
proc-joint = Siūlės
proc-second = Antra spalva
proc-count = Elementų skersai × aukštyn
proc-scale = Mastelis
proc-joint-width = Siūlės plotis
proc-variation = Įvairovė
proc-seed = Variantas
proc-another = Kitas
proc-width = Plytelės plotis
proc-name = Pavadinimas
proc-hint = Plytelė kartojasi be siūlių; peržiūroje – 2 × 2 plytelės.
proc-create = Sukurti medžiagą
proc-plugin-hover = Raštas iš įskiepio (@texture_generator)
proc-generating = Generuojama…
proc-created = Sukurta medžiaga { $name }: spustelėkite sienas, kad nudažytumėte
mat-import = Importuoti…
mat-import-hover = Pridėti medžiagų failus (.vrstmat, papildinių formatai) prie dokumento
mat-choose-folder = Pasirinkti aplanką…
mat-default-folder = Numatytasis aplankas
mat-default-folder-hover = Atgal į { DATA_DIR() }/materials
mat-purge = Pašalinti nenaudojamas medžiagas
mat-purge-hover = Pašalinti medžiagas, kurių modelyje niekas nenaudoja
mat-show-examples-hover = Kartu su Varstotas pateikiamos medžiagos (demonstracija „Medžiagų galerija“)
mat-in-model-hover = Atidaryto dokumento medžiagos
mat-favourites-hover = Spustelėkite bibliotekos medžiagą dešiniuoju mygtuku → Pridėti prie mėgstamiausių
mat-recent-hover = Paskutinės naudotos bibliotekos medžiagos
mat-examples-hover = Kartu su Varstotas pateikiamos medžiagos (demonstracija „Medžiagų galerija“)
mat-empty = Tuščia: spustelėkite modelio medžiagą dešiniuoju mygtuku → Išsaugoti bibliotekoje
mat-use = Naudoti
mat-use-hover = Dažyti ja (pridedama prie modelio, kai kas nors nudažoma)
mat-export = Eksportuoti…
mat-save-hover = Įrašyti į medžiagų bibliotekos aplanką, kad būtų prieinama bet kuriam dokumentui
mat-copy-hover = Nukopijuoti į jūsų medžiagų biblioteką
mat-delete-model-hover = Pašalinti iš modelio: tai, kas ja nudažyta, gauna numatytąją medžiagą (Atšaukti ją sugrąžina)
mat-replace = Pakeisti dabartine medžiaga
mat-replace-hover = Perdažyti viską, kas nudažyta šia medžiaga, dabartine medžiaga ir pašalinti ją iš modelio (Atšaukti ją grąžins)
mat-undo-replace = Pakeisti { $name } į { $with }
mat-replaced = { $name } → { $with }: perdažyta { $n }
mat-file-filter = Varstotas medžiaga
mat-all-filter = Visos medžiagos

## Tile description (hover)

mat-kind-metal = metalas
mat-kind-glass = stiklas
mat-kind-emissive = švytinti
mat-kind-texture = tekstūra
# $name is the shader preset.
mat-kind-shader = šešėliuoklė { $name }
mat-kind-shader-custom = pasirinktinė
# $value is the roughness, 0 to 1.
mat-kind-roughness = šiurkštumas { $value }

## Messages

mat-undo-add = Pridėti medžiagą { $name }
mat-undo-delete = Naikinti medžiagą { $name }
mat-undo-rename = Pervadinti medžiagą { $name }
mat-saved = Išsaugota medžiagų bibliotekoje: { $name }
mat-save-failed = Nepavyko išsaugoti medžiagos: { $error }
mat-exported = Eksportuota { $path }
mat-export-failed = Nepavyko eksportuoti medžiagos: { $error }
# $names are the imported materials, $plugin the importer.
mat-imported-with = Importuota { $names } naudojant { $plugin }
mat-import-empty = { $plugin }: medžiagų faile { $path } nėra
mat-import-failed-with = Nepavyksta importuoti { $path } naudojant { $plugin }: { $error }
mat-imported = Importuota medžiaga { $name }
mat-import-failed = Nepavyksta importuoti { $path }: { $error }
# $n is the number of painted items put back to the default material.
mat-deleted-uses = Ištrinta medžiaga { $name }, naudojimų grąžinta į numatytąją: { $n } (Atšaukti ją sugrąžina)
mat-deleted = Ištrinta medžiaga { $name } (Atšaukti ją sugrąžina)
mat-delete-failed = Nepavyksta ištrinti medžiagos { $name }
mat-rename-exists = Medžiaga „{ $name }“ jau yra
mat-file-exists = { $path } jau yra
mat-delete-title = Naikinti iš medžiagų bibliotekos?
mat-delete-undone = Šio veiksmo atšaukti negalima. Ją naudojantys dokumentai išsaugo savo kopiją.
mat-deleted-from-library = Ištrinta iš medžiagų bibliotekos: { $name }

## Online materials (window and catalogues)

mat-online-title = Internetinė medžiagų biblioteka
mat-online-plugins-hint =
    Papildiniai su @material_source
    rodomi čia
mat-online-search = Paieška: plyta, ąžuolas, betonas…
mat-online-description = Nemokami CC0 nuotraukų skenavimai (spalvos, normalių, šiurkštumo, okliuzijos, metalo žemėlapiai realiu dydžiu). Atsisiuntimai patenka į medžiagų biblioteką.
mat-online-click-hint = Spustelėkite medžiagą, kad ją pamatytumėte; dukart spustelėjus ji atsisiunčiama ir ja dažoma.
# $width is a length.
mat-online-wide = plotis { $width }
mat-online-use-hover = Jau bibliotekoje: dažyti ja
mat-online-download-use = Atsisiųsti ir naudoti
mat-online-download-use-hover = Išsaugoti bibliotekoje, pridėti prie modelio ir dažyti
mat-online-download-hover = Išsaugoti medžiagų bibliotekoje
# $source is the source's name.
mat-online-loading = Įkeliamas katalogas { $source }…
mat-online-load-failed = Nepavyksta įkelti katalogo: { $error }
# $n is the number of shown materials.
mat-online-count = Medžiagų: { $n }
mat-online-load-hover = Paprašyti papildinio medžiagų su šiomis reikšmėmis
mat-online-downloaded = { $name } atsisiųsta į medžiagų biblioteką
mat-terms-polyhaven = Teikia Poly Haven (polyhaven.com). Tekstūros yra CC0: nemokamos bet kokiam naudojimui, taip pat komerciniam; nurodyti autorystės nereikia.
mat-terms-ambientcg = ambientCG medžiagos yra CC0: nemokamos bet kokiam naudojimui, taip pat komerciniam; autorystės nurodyti nebūtina.
mat-err-not-catalogue = tai ne katalogas
mat-err-no-colour-map = nėra spalvų žemėlapio

## Medžiagos redaktorius
mated-window-title = Medžiagos redaktorius
mated-none = Nėra medžiagos: pasirinkite pavyzdį ar bibliotekos medžiagą arba paimkite ją iš pažymėjimo.
mated-not-in-model = Dar ne modelyje: bus pridėta, kai ja nudažysite arba ją pakeisite.
mated-used = { $count ->
        [one] Naudojama { $count } paviršiuje ar objekte
       *[other] Naudojama paviršiuose ir objektuose: { $count }
    }
mated-pick = Paimti iš pažymėjimo
mated-pick-tip = Pipetė: pažymėto paviršiaus ar objekto medžiagą padaryti dabartine
mated-pick-none = Pažymėkite nudažytą paviršių ar objektą, kad paimtumėte jo medžiagą
mated-picked = Dabartinė medžiaga: { $name }
mated-select-tip = Pažymėti šio konteksto paviršius ir objektus, nudažytus šia medžiaga
mated-selected = { $count ->
        [one] Pažymėtas { $count } elementas
        [few] Pažymėti { $count } elementai
        [many] Pažymėta { $count } elemento
       *[other] Pažymėta { $count } elementų
    }
mated-window-tip = Atidaryti redaktorių lange
mated-basic = Pagrindinės
mated-color = Spalva
mated-opacity = Nepermatomumas
mated-texture = Tekstūra
mated-no-texture = Be tekstūros
mated-look = Išvaizda
mated-hue = Atspalvis
mated-saturation = Sodrumas
mated-lightness = Šviesumas
mated-cutoff = Permatomumo slenkstis
mated-colorize = Tonuoti
mated-colorize-tip = Nuspalvinti vaizdą medžiagos spalva, išlaikant jo šviesumą
mated-decal = Lipdukas
mated-decal-tip = Rodyti vieną plytelę be kartojimo: aplink ją matoma medžiagos spalva
mated-world = Pasaulio koordinatės
mated-world-tip = Koordinatės pagal padėtį pasaulyje (kubinė projekcija): nepaiso tekstūros padėties, raštas tęsiasi per sienas
mated-look-reset = Atkurti išvaizdą
mated-undo-look = Tekstūros išvaizda
mated-undo-color = Keisti medžiagos spalvą
mated-textures = Tekstūros
mated-slot-base = Pagrindinė tekstūra
mated-slot-empty = Tuščia
mated-view-tip = Peržiūrėti dideliu mastu
mated-downscale = Sumažinti
mated-downscale-tip = Didelis vaizdas: atvaizdavimas naudoja ne daugiau 1024 px. Perkoduoti į { $side } px, kad dokumentas būtų mažesnis
mated-load-tip = Įkelti vaizdą…
mated-replace-tip = Pakeisti kitu vaizdu… (plytelės dydis išlieka)
mated-clear-tip = Pašalinti šią tekstūrą
mated-export-tip = Išsaugoti vaizdą faile…
mated-edit-tip = Redaguoti numatytajame failo vaizdų redaktoriuje: išsaugojus ten, medžiaga atnaujinama
mated-map-set = Įkelti žemėlapių rinkinį iš aplanko…
mated-map-set-tip = Užpildyti lizdus iš aplanko pagal failų vardų priesagas: _color / _albedo / _diff, _normal (geriau OpenGL), _rough, _metal, _ao, _emission
mated-watching = { $count ->
        [one] Stebima { $count } tekstūra, atidaryta redaktoriuje
        [few] Stebimos { $count } tekstūros, atidarytos redaktoriuje
        [many] Stebima { $count } tekstūros, atidarytos redaktoriuje
       *[other] Stebima { $count } tekstūrų, atidarytų redaktoriuje
    }
mated-view-title = { $material }: { $slot }
mated-view-tiled = Kartoti 3 × 3
mated-undo-texture = Įkelti tekstūrą
mated-undo-map-set = Įkelti žemėlapių rinkinį
mated-undo-clear = Pašalinti tekstūrą
mated-undo-downscale = Sumažinti tekstūrą
mated-undo-reload = Iš naujo įkelti tekstūrą
mated-no-maps = { $path } nėra vaizdų su žemėlapių priesagomis
mated-maps-loaded = { $count ->
        [one] Įkeltas { $count } žemėlapis
        [few] Įkelti { $count } žemėlapiai
        [many] Įkelta { $count } žemėlapio
       *[other] Įkelta { $count } žemėlapių
    }
mated-exported = Išsaugota: { $path }
mated-editing = Redaguojama { $file }: išsaugokite, kad atnaujintumėte medžiagą
mated-cant-open-editor = Nepavyko atidaryti vaizdų redaktoriaus: { $error }
mated-reloaded = { $slot }: įkelta iš naujo iš vaizdų redaktoriaus

## ======== Internetinė 3D biblioteka ========

model-lib-licence-any = Bet kuri licencija
model-lib-licence-commercial = Komercinis naudojimas leidžiamas
model-lib-licence-cc0 = Tik CC0
model-lib-licence-known = Žinoma licencija

## Window and filters

model-lib-title = Internetinė 3D biblioteka
model-lib-source = Šaltinis
model-lib-source-count = { $label } ({ $n })
model-lib-plugins-hint =
    Papildiniai su @model_source
    rodomi čia
model-lib-search-enter = Ieškokite ir spauskite Enter: kėdė, kaukolė…
model-lib-search = Paieška: kėdė, uola, augalas…
model-lib-all-categories = Visos kategorijos
# $description is the source's description.
model-lib-downloads-go = { $description } Atsisiuntimai patenka į komponentų biblioteką.
model-lib-from-url = Iš URL
model-lib-import = Importuoti
model-lib-import-hover = Atsisiųsti failą į komponentų biblioteką ir įdėti
model-lib-click-hint = Spustelėkite modelį, kad jį pamatytumėte; dukart spustelėjus jis atsisiunčiamas ir įdedamas.
model-lib-type-search = Įveskite paieškos frazę ir paspauskite Enter.
model-lib-loading = Įkeliami { $source } modeliai…
model-lib-load-failed = Nepavyksta įkelti modelių: { $error }
model-lib-retry = Bandyti dar kartą
# $n is the number of shown models.
model-lib-count = Modelių: { $n }
model-lib-show = Rodyti
model-lib-format-any = Bet koks formatas
model-lib-size-any = Bet koks dydis
model-lib-size-10k = ≤ 10 tūkst. trikampių
model-lib-size-50k = ≤ 50 tūkst. trikampių
model-lib-size-100k = ≤ 100 tūkst. trikampių
model-lib-clear = Išvalyti
model-lib-reload = Įkelti iš naujo
model-lib-load = Įkelti
model-lib-choose-with = pasirinkti naudojant …
model-lib-choose-it = Pasirinkti
model-lib-plugin-apply-hover = Naudoti šias reikšmes (tada ieškoti)
model-lib-plugin-load-hover = Paprašyti papildinio modelių su šiomis reikšmėmis

## Details of a model

# $n is a number of triangles.
model-lib-triangles = trikampių: { $n }
# $value is the size in megabytes.
model-lib-megabytes = { $value } MB
model-lib-heavy = ⚠ Didelis modelis: gali sulėtinti rodinį.
model-lib-downloading = Atsisiunčiama…
# $n is the number of models of the pack in the library.
model-lib-in-components = ✔ Modelių komponentuose: { $n }
model-lib-show-in-components = Rodyti komponentuose
model-lib-download-browser = Atsisiųsti naršyklėje
model-lib-download-browser-hover = Atverti paketo atsisiuntimą naršyklėje; archyvas importuojamas iš Atsisiuntimų kartu su autorystės nuoroda
model-lib-download-again = Atsisiųsti iš naujo
model-lib-download-pack = Atsisiųsti paketą
model-lib-download-pack-hover = Atsisiųsti paketą: kiekvienas modelis tampa bibliotekos komponentu
model-lib-place-hover = Jau komponentų bibliotekoje: įdėti į modelį
model-lib-download-place = Atsisiųsti ir įdėti
model-lib-download-place-hover = Išsaugoti komponentų bibliotekoje ir įdėti į modelį
model-lib-download = Atsisiųsti
model-lib-download-hover = Išsaugoti komponentų bibliotekoje
model-lib-web-page = Tinklalapis

## Messages

# $name is the model.
model-lib-cancelled = { $name } atsisiuntimas atšauktas
model-lib-fav-added = ☆ { $name } pridėtas prie mėgstamiausių
model-lib-fav-removed = { $name } pašalintas iš mėgstamiausių
model-lib-paste-url = Įklijuokite modelio failo adresą (https://…/model.glb)
model-lib-browser-download = Atsisiųskite { $name } naršyklėje: Varstotas importuoja archyvą iš Atsisiuntimų
# $n is the number of models, $folder the library folder, $pack the pack's folder.
model-lib-downloaded-pack = Modelių atsisiųsta: { $n } į Komponentai → { $folder } → { $pack }
model-lib-downloaded = Atsisiųstas { $name } (paviršių: { $faces }) į Komponentai → { $folder }
model-lib-not-in-library = { $path } nėra komponentų bibliotekoje
model-lib-download-failed = Atsisiųsti nepavyko: { $error }
model-lib-import-failed = Importuoti { $path }: { $error }
model-lib-undo-import = Importuoti { $name }
model-lib-imported = Importuota { $path }

## Sources (online_models)

model-lib-source-favourites = Mėgstamiausi
model-lib-source-downloaded = Atsisiųsti
model-lib-terms-polyhaven = Teikia Poly Haven (polyhaven.com). Modeliai yra CC0: nemokami bet kokiam naudojimui, taip pat komerciniam; nurodyti autorystės nereikia.
model-lib-terms-smithsonian = Dauguma objektų yra CC0, bet kai kurie pažymėti „Taikomos naudojimo sąlygos“ (tik nekomerciniam naudojimui). Prieš naudodami komerciškai patikrinkite objekto puslapį; prekių ženklai ir kitos pačių objektų teisės neapimamos.
model-lib-terms-nasa = NASA medžiagos paprastai nėra saugomos autorių teisių, bet modeliuose esantys NASA ženklai ir logotipai yra apsaugoti, ir niekas negali sudaryti įspūdžio, kad NASA remia jūsų produktą.
model-lib-terms-kenney = Kenney paketai yra CC0: nemokami bet kokiam naudojimui, taip pat komerciniam; autorystės nurodyti nebūtina. Kenney logotipas į tai neįeina.
model-lib-terms-quaternius = Quaternius paketai yra CC0: nemokami bet kokiam naudojimui, taip pat komerciniam; autorystės nurodyti nebūtina.
model-lib-terms-saved = Kiekvienas modelis išlaiko licenciją, su kuria buvo gautas (jo informacijoje; Failas → Eksportuoti → Autorystė). Varstotas perspėja, kai įdėto ar eksportuojamo modelio licencija riboja jo naudojimą.
model-lib-desc-polyhaven = Nemokami CC0 modeliai (glTF, 1K tekstūros) tikruoju dydžiu.
model-lib-desc-smithsonian = Smithsonian Open Access 3D: muziejaus eksponatų skenavimai. Įveskite paieškos frazę ir paspauskite Enter.
model-lib-desc-nasa = NASA 3D ištekliai: erdvėlaiviai, marsaeigiai, planetos (GLB).
model-lib-desc-kenney = Kenney žaidimų išteklių paketai (CC0): paketas atsisiunčiamas visas, kiekvienas modelis tampa komponentu skiltyje Komponentai → Kenney → <paketas>.
model-lib-desc-quaternius = Quaternius mažo poligonų skaičiaus paketai (CC0). Jie atsisiunčiami jūsų naršyklėje (Google Drive): Varstotas atveria puslapį ir importuoja archyvą iš Atsisiuntimų.
model-lib-desc-favourites = Modeliai, kuriuos pažymėjote ☆ (spustelėkite ☆ modelio informacijoje).
model-lib-desc-history = Viskas, kas atsisiųsta arba importuota iš Atsisiuntimų, naujausi viršuje. Rašykite, kad ieškotumėte.

## Errors of catalogues and imports

model-lib-err-not-catalogue = tai ne katalogas
model-lib-err-not-search = tai ne paieškos atsakymas
model-lib-err-kenney-packs = kenney.nl paketų nerasta (puslapis pasikeitė?)
model-lib-err-quaternius-packs = quaternius.com paketų nerasta (puslapis pasikeitė?)
model-lib-err-plugin-array = papildinys turi grąžinti modelių masyvą
model-lib-err-no-download = paketo puslapyje nėra atsisiuntimo
model-lib-err-quaternius-browser = Quaternius paketai atsisiunčiami naršyklėje: naudokite Atsisiųsti naršyklėje
model-lib-err-importer-depth = importavimo papildiniai kviečia vienas kitą per giliai
model-lib-err-empty-answer = tuščias atsakymas
model-lib-err-plugin-nothing = papildinys nieko negrąžino
model-lib-err-spec-object = modelio aprašas turi būti objektas
model-lib-err-spec-empty = modelio apraše nėra failo, url, zip ar tinklelių (meshes)
model-lib-err-archive-no-model = archyve nėra modelio failo (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf)
# $k is the number of the mesh in the answer.
model-lib-err-mesh-vertices = tinklelis { $k }: nėra viršūnių
model-lib-err-mesh-faces = tinklelis { $k }: nėra paviršių
model-lib-err-no-faces = tinkleliuose nėra paviršių
model-lib-err-model-empty = modelis tuščias
model-lib-err-no-size = modelis neturi dydžio
model-lib-err-no-home = nėra namų aplanko
model-lib-err-archive-none = archyve nėra modelio failų
model-lib-err-none-read = nepavyko perskaityti jokių modelių

## ======== Atsisiuntimai ========

## Pranešimai
# $path is a file.
downloads-no-models = { $path }: jame nėra modelių failų
downloads-no-address = { $path }: jame nėra interneto adreso
downloads-not-model = { $path }: ne modelio failas (glb, gltf, fbx, dae, obj, 3mf, ply, stl, dxf, varstotas, zip)
# $site is the site, $title the model.
downloads-waiting-site = Laukiama failo iš { $site }: atsisiųskite jį naršyklėje
downloads-waiting-file = Laukiama modelio „{ $title }“ failo ({ $site }): atsisiųskite jį naršyklėje
downloads-no-library = Nėra komponentų bibliotekos aplanko
# $name is the file, $folder the library folder, $n the number of models.
downloads-imported = Importuota { $name } į Komponentai → { $folder }
downloads-imported-many = Importuota modelių: { $n } iš { $name } į Komponentai → { $folder }
downloads-import-failed = Importas { $name }: { $error }
downloads-err-no-file-name = nėra failo pavadinimo
downloads-err-no-model-chosen = nepasirinktas modelis
# $label is a licence, $names the first models under it, $more " +N" or empty, $note what it asks.
downloads-licence-part = { $label } ({ $names }{ $more }) — { $note }
# $list is the licence-part texts.
downloads-licences = Licencijos: { $list }. Failas → Eksportuoti → Priskyrimai pateikia autorių nuorodas.

## Paieška, kitos svetainės

downloads-search-on = Ieškoti:
# $site is the site, $query the search.
downloads-search-hover = Atidaryti { $site } paiešką pagal „{ $query }“ naršyklėje
downloads-edit = Redaguoti…
downloads-edit-hover = Redaguoti svetaines ({ DATA_DIR() }/sources.toml); sąrašas perskaitomas iš naujo
downloads-other = Kitos svetainės
# $waiting is the number of waiting downloads.
downloads-other-waiting = Kitos svetainės — laukiama atsisiuntimų: { $waiting }
downloads-other-text = Atsisiųskite iš bet kurios svetainės savo naršyklėje: Varstotas pats svetainių nelanko. Jis paima failą iš Atsisiuntimų aplanko ir išsaugo puslapį kaip autorystės nuorodą.
downloads-watch = Stebėti Atsisiuntimus
downloads-watch-hover = Siūlyti importuoti kiekvieną naują modelio failą ar archyvą iš Atsisiuntimų aplanko (visada įjungta, kol laukiama puslapio)
# $dir is the watched folder.
downloads-watching = stebimas { $dir }
downloads-off = išjungta
downloads-import-file = Importuoti failą…
downloads-import-file-hover = Importuoti atsisiųstą modelio failą ar archyvą
downloads-models-filter = Modeliai
# $title is the model, $site the site, $licence its licence.
downloads-waiting-card = Laukiama failo iš šio puslapio: { $title } ({ $site }, { $licence })
downloads-page = Puslapis
# $file is a file address.
downloads-browser-file = Atsisiųsti naršyklėje: { $file }
downloads-stop = Nebelaukti

## Atsisiuntimo importo langas

downloads-import-title = Importuoti atsisiuntimą
downloads-importing = Importuojama…
# $name is the file, $mb its size in megabytes.
downloads-file-size = { $name }  ({ $mb } MB)
downloads-credit-from-page = ✔ Autorystės nuoroda paimta iš jūsų atsiųsto puslapio
downloads-duplicate = ⚠ Šis pats failas jau buvo importuotas
downloads-field-title = Pavadinimas
downloads-field-author = Autorius
downloads-field-site = Svetainė
downloads-licence = Licencija
downloads-licence-hint = kaip nurodyta puslapyje
downloads-place = Įdėti į modelį
# $on models are chosen of $n.
downloads-models-count = Pasirinkta modelių: { $on } iš { $n } (kiekvienas tampa komponentu)
downloads-all = Visi
downloads-none = Nė vieno
# $folder is the library folder of the site.
downloads-destination = Komponentai → { $folder }
downloads-ignore = Praleisti
# $n is the number of other offers.
downloads-more-waiting = Laukia dar: { $n }

## ======== Atvaizdavimas ========

render-dof-view = Kaip rodinyje
render-dof-off = Išjungta
render-dof-on = Įjungta

## Kokybės išankstiniai nustatymai ir vaizdo dydžiai (id lieka angliški)

render-preset-draft = Juodraštis
render-preset-preview = Peržiūra
render-preset-final = Galutinis
render-preset-best = Geriausias
render-size-hd = HD 1280 × 720
render-size-full-hd = Full HD 1920 × 1080
render-size-qhd = QHD 2560 × 1440
render-size-4k = 4K 3840 × 2160
render-size-square = Kvadratas 1080
render-size-portrait = Portretas 1080 × 1350

## Mėnesiai (saulės data)

render-month-jan = Saus.
render-month-feb = Vas.
render-month-mar = Kov.
render-month-apr = Bal.
render-month-may = Geg.
render-month-jun = Birž.
render-month-jul = Liep.
render-month-aug = Rugp.
render-month-sep = Rugs.
render-month-oct = Spal.
render-month-nov = Lapkr.
render-month-dec = Gruod.

## Eksporto ir šešėliuoklių klaidos

render-export-empty = vaizdo dydis turi būti bent 1 x 1 taškas
# $w and $h are the requested size, $max the limit per side.
render-export-too-large = vaizdo dydis { $w } x { $h } per didelis (daugiausia { $max } taškų vienoje pusėje)
render-export-gpu = GPU klaida eksportuojant vaizdą: { $error }
render-shader-clash = nesikompiliuoja kartu su kitomis šešėliuoklėmis: { $error }

## Atvaizdavimo langas

render-title = Atvaizdavimas
render-current-view = Atvaizduoti dabartinį rodinį
render-stop = Stabdyti
render-continue = Tęsti
render-more-samples = Daugiau pavyzdžių
render-more-samples-tip = Padvigubinti pavyzdžių skaičių
render-save-png = Išsaugoti PNG…
render-save-exr = Išsaugoti EXR…
render-save-exr-tip = Tiesinė šviesa, 32 bitų slankiojo kablelio
render-all-scenes = Atvaizduoti visas scenas…
# $error is the reason.
render-failed = Atvaizdavimas nepavyko: { $error }
render-error = Atvaizdavimas: { $error }
# $n is the number of scenes, $folder where they were saved.
render-scenes-done = Scenų atvaizduota: { $n }, aplanke { $folder }
# $written scenes are saved, scene $next is being rendered.
render-scenes-progress = Atvaizduojamos scenos: išsaugota { $written }, vykdoma scena { $next }
# $path is the saved file.
render-saved = Išsaugota { $path }
# $secs is a number with one decimal.
render-status-running = { $samples } / { $max } pavyzdžių · { $secs } s
# $denoised is "yes" or "no".
render-status-done = Baigta: { $samples } pavyzdžių per { $secs } s{ $denoised ->
        [yes] , triukšmas pašalintas
       *[no] {""}
    }
render-status-stopped = Sustabdyta ties { $samples } pavyzdžių · { $secs } s
render-undo-lights = Redaguoti šviesas
render-undo-settings = Atvaizdavimo nustatymai

## Šviesos

render-lights = Šviesos ({ $n })
render-add-point = + Taškinė
render-add-spot = + Prožektorius
render-add-area = + Ploto
render-on = Įjungta
render-light-point = taškinė
render-light-spot = prožektorius
render-light-area = ploto
render-light-name = Pavadinimas
render-light-type = Tipas
render-light-position = Padėtis
render-light-to-centre = Į rodinio centrą
render-light-to-centre-tip = Perkelti 2 m virš kameros taikinio
render-light-to-camera = Į kamerą
render-light-aim-at-target = Nukreipti į taikinį
render-light-aim-at-target-tip = Šviesti į rodinio taikinį
render-light-aim = Nukreipimas
render-colour = Spalva
render-light-power = Galia
render-light-radius = Spindulys
render-light-radius-tip = Lemputės dydis: švelnesni šešėliai
render-light-cone = Kūgis
render-light-cone-angle = Kūgio kampas
render-light-soft = Švelnumas
render-light-soft-edge = Švelnus kraštas
render-size = Dydis

## Nustatymai

render-sec-image = Vaizdas
render-sec-sampling = Mėginių ėmimas
render-sec-camera = Kamera
render-sec-light = Šviesa
render-sec-colour = Spalva
render-quality = Kokybė
# $samples and $bounces are numbers.
render-quality-tip = pavyzdžių: { $samples }, atspindžių: { $bounces }
render-match-view = Pritaikyti prie rodinio
render-match-view-tip = Rodinio proporcijos šiame aukštyje
render-size-custom = Pasirinktinis
render-transparent = Skaidrus fonas
render-transparent-tip = Dangus ir žemė už modelio PNG faile skaidrūs (jie vis tiek jį apšviečia)
render-samples = Pavyzdžiai
render-time-limit = Laiko riba
render-time-limit-tip = 0 = kol bus surinkti pavyzdžiai
render-bounces = Atšokimai
render-clamp = Apribojimas
render-clamp-tip = Ryškiausia netiesioginė pavyzdžio šviesa: mažesnė reikšmė pašalina ryškius taškus, didesnė išlaiko ryškią atspindėtą šviesą; 0 = išjungta
render-denoiser = Triukšmo šalinimas
render-denoiser-off = Išjungta
render-denoiser-builtin = Integruotas
render-denoiser-builtin-tip = Kraštus išlaikantis filtras
render-denoiser-ai = DI (Open Image Denoise)
render-denoiser-install = Įdiekite Open Image Denoise: brew install open-image-denoise
render-dof = Lauko gylis
render-dof-tip = Kaip rodinyje: Efektų lauko gylis, kai jis įjungtas
# $mm is the equivalent focal length.
render-lens = { $mm } mm objektyvas
render-lens-tip = Rodinio matymo laukas 35 mm kameroje
render-aperture = Diafragma
render-aperture-tip = Mažesnis f skaičius = siauresnis ryškumo plotas
render-focus = Fokusas
render-focus-target = Taikinys
render-focus-target-tip = Fokusuoti į rodinio taikinį (sukimosi centrą)
render-sun = Saulė
render-sun-tip = Saulės stiprumas (jos kryptis: Šešėliai)
render-sky = Dangus
render-sky-tip = Aplinkos šviesa
render-rotation = Pasukimas
render-rotation-tip = Sukioja dangų / HDRI
render-exposure = Ekspozicija
render-env-exposure-tip = Aplinkos ekspozicija (rodinys ir atvaizdavimas)
render-exposure-tip = Pridedama prie aplinkos ekspozicijos (tik atvaizdavimui)
render-tone-curve = Tonų kreivė
# $name is the environment's tone curve.
render-tone-environment = Aplinka: { $name }
render-white-balance = Baltos spalvos balansas
render-white-balance-tip = 6500 K = neutralus; didesnė reikšmė šiltesnė
render-tint = Atspalvis
render-tint-tip = Žalia (−) .. purpurinė (+)
render-saturation = Sodrumas
render-contrast = Kontrastas
render-vignette = Vinjetė
render-reset = Atkurti
render-reset-tip = Grįžti prie numatytųjų spalvų nustatymų
render-shader-define-surface = define fn surface(s: Surface, p: Params) -> Surface
# $line is the line in the user's shader code, $msg the compiler's message.
render-shader-line = eilutė { $line }: { $msg }

## ======== Realistinis stilius ========

realistic-undo-material = Redaguoti medžiagą
# $path is a file, $error the reason.
realistic-cant-use = Negalima naudoti { $path }: { $error }
realistic-cant-read = Negalima perskaityti { $path }: { $error }
realistic-used-by = Naudoja Realistinis stilius
realistic-load = Įkelti…
realistic-clear = Išvalyti
realistic-reset = Atkurti
realistic-preset = Šablonas

## Aplinka

realistic-background = Fonas
realistic-sky-sky = Dangus
realistic-sky-color = Spalva
realistic-sky-hdri = HDRI
realistic-sky-physical = Fizinis dangus
realistic-sky-physical-tip = Dienos šviesa pagal saulės padėtį Šešėlių kortelėje
realistic-haze = Migla
realistic-haze-tip = Drumstumas: 2 = giedras mėlynas dangus, 10 = miglotas; saulės laikas ir data yra Šešėlių kortelėje
realistic-hdri-none = nėra (dangus)
realistic-hdri-tip = Ekvirectangulinė panorama (.hdr)
realistic-rotation = Pasukimas
realistic-intensity = Intensyvumas
realistic-sun = Saulė
realistic-sun-tip = Saulės stiprumas; jos kryptis imama iš Šešėlių kortelės, kai šešėliai įjungti
realistic-exposure = Ekspozicija
realistic-tone-mapping = Tonų atvaizdavimas
realistic-tonemap-aces = ACES (kino)
realistic-tonemap-agx = AgX (švelnus)
realistic-tonemap-none = Nėra
realistic-show-env = Rodyti aplinką už modelio
realistic-effects = Efektai

## Fizinės savybės

realistic-physical = Fizinės (Realistinis)
realistic-preset-matte = Matinė
realistic-preset-satin = Satininė
realistic-preset-glossy = Blizgi
realistic-preset-metal = Metalas
realistic-preset-chrome = Chromas
realistic-preset-glass = Stiklas
realistic-preset-light = Šviesa
realistic-metallic = Metališkumas
realistic-metallic-tip = 0 = dažai, plastikas, mediena; 1 = švarus metalas
realistic-roughness = Šiurkštumas
realistic-roughness-tip = 0 = lyg veidrodis, 1 = matinis
realistic-glass = Stiklas
realistic-glass-tip = Kiek šviesos praleidžia
realistic-ior = IOR
realistic-ior-tip = Lūžio rodiklis (stiklo 1,5, vandens 1,33)
realistic-emission = Spinduliavimas
realistic-emission-tip = Paviršiaus skleidžiama šviesa: spalva ir stiprumas
realistic-coat-sheen = Danga, Blizgesys, Kraštai
realistic-clearcoat = Skaidri danga
realistic-clearcoat-tip = Blizgus lako sluoksnis: lakuota mediena, automobilio dažai, glazūruotos plytelės
realistic-coat-rough = Dangos šiurkštumas
realistic-coat-rough-tip = Lako šiurkštumas (0 = blizgus)
realistic-sheen = Blizgesys
realistic-sheen-tip = Švelnus švytėjimas šliaužiančiais kampais: aksomas, audinys, apmušalai
realistic-sheen-tint = Blizgesio atspalvis
realistic-sheen-tint-tip = 0 = baltas blizgesys, 1 = medžiagos spalva
realistic-anisotropy = Anizotropija
realistic-anisotropy-tip = Ištęsti atspindžiai išilgai tekstūros (+) arba skersai (−): šlifuotas metalas
realistic-anti-tiling = Prieš kartojimąsi
realistic-anti-tiling-tip = Sumažina matomą tekstūrų kartojimąsi
realistic-bevel = Nukirtimas, mm
realistic-bevel-tip = Suapvalina paviršių briaunas atvaizduojant (tik spindulių sekimas)
realistic-shader = Šešėliuoklė
realistic-shader-tip = WGSL paviršiaus programa (docs/SHADERS.md): procedūriniai raštai, bangos, švytėjimas
realistic-shader-none = Nėra
realistic-shader-custom = Pasirinktinė
realistic-shader-edit = Redaguoti…
realistic-shader-edit-tip = Parašyti ar pakeisti šešėliuoklę
realistic-map-normal = Normalių
realistic-map-roughness = Šiurkštumo
realistic-map-metallic = Metališkumo
realistic-map-occlusion = Užtemdymo
realistic-map-emission = Spinduliavimo
realistic-normal-strength = Normalių stiprumas
realistic-remove-physical = Pašalinti fizines savybes

## Šešėliuoklių redaktorius

# $material is the material's name.
realistic-shader-window = Šešėliuoklė — { $material }
realistic-start-from = Pradėti nuo…
realistic-start-from-tip = Pakeisti kodą šia šešėliuokle
realistic-checker-new = Šachmatinis (naujas)
realistic-reference = Žinynas
realistic-reference-tip = Paviršiaus laukai, pagalbinės funkcijos ir integruotosios funkcijos
realistic-compiles = Kompiliuojasi; medžiaga ją naudoja
realistic-ref-surface = Paviršius (s, o)
realistic-ref-parameters = Parametrai
realistic-ref-functions = Funkcijos

## Efektai

realistic-fx-off = Išjungta
realistic-fx-fast = Greitas
realistic-fx-balanced = Subalansuotas
realistic-fx-quality = Kokybiškas
realistic-fx-soft-shadows = Švelnūs šešėliai
realistic-fx-soft-shadows-tip = Šešėliai išsilieja tolstant nuo juos metančio objekto
realistic-fx-sun-size = Saulės dydis
realistic-fx-ao = Aplinkos užtemdymas
realistic-fx-ao-tip = Tamsesni įtrūkimai ir sąlyčio šešėliai
realistic-fx-radius = Spindulys, mm
realistic-fx-strength = Stiprumas
realistic-fx-reflections = Atspindžiai
realistic-fx-reflections-tip = Modelis, atsispindintis blizgiuose paviršiuose (ekrano erdvėje)
realistic-fx-bloom = Žydėjimas
realistic-fx-bloom-tip = Ryški šviesa švyti
realistic-fx-threshold = Slenkstis
realistic-fx-dof = Lauko gylis
realistic-fx-dof-tip = Suliejimas prieš fokuso atstumą ir už jo
realistic-fx-blur = Suliejimas
realistic-fx-focus = Fokusas, mm
realistic-fx-focus-tip = 0 = taškas rodinio centre
realistic-fx-fog = Rūkas
realistic-fx-fog-tip = Migla, stiprėjanti didėjant atstumui
realistic-fx-fog-distance = Atstumas, mm
realistic-fx-fog-height = Aukštis, mm
realistic-fx-fog-height-tip = 0 = tolygus rūkas; kitaip jis retėja per šį aukštį
realistic-vignette = Vinjetė
realistic-style = Stilius
realistic-stylize-photo = Nuotrauka
realistic-stylize-pencil = Pieštukas
realistic-stylize-watercolor = Akvarelė
realistic-stylize-technical = Techninis
realistic-fx-clay = Molis
realistic-fx-clay-tip = Kiekviena medžiaga kaip paprastas molis (apimties modelis)
realistic-fx-high-quality = Aukšta kokybė
realistic-fx-high-quality-tip = Daugiau pavyzdžių švelniems šešėliams, užtemdymui ir atspindžiams

## ======== Animacija ========

anim-title = Eksportuoti animaciją
anim-no-scenes = animacijoje nėra scenų
anim-no-scenes-flash = Animacijoje nėra scenų
# $error is the reason.
anim-error = Animacija: { $error }
# $what is a file path, or "N frames in <folder>".
anim-saved = Išsaugota { $what }
anim-render-dropped = atvaizdavimas nutrauktas
# $n is the frame count, $dir the folder with the PNG frames.
anim-frames-in = Kadrų: { $n }, aplanke { $dir }
anim-cancelled = atšaukta
anim-animate = Animuoti
anim-scenes = Scenos
anim-add-scenes = Pirmiausia pridėkite scenų (su „Įtraukti į animaciją“)
anim-orbit = Skrieti aplink rodinį
anim-length = Trukmė
anim-turns-suffix = {" "}apsisukimų
anim-holds-hint = Pauzės ir perėjimai: Scenų kortelė → Animacija
anim-size = Dydis
anim-fps = Kadrų dažnis
anim-frames = Kadrai
anim-as-view = Kaip rodinyje
anim-path-traced = Spindulių sekimas
anim-output = Išvestis
anim-gif = Animuotas GIF
anim-png-sequence = PNG seka
# $done of $total frames.
anim-frame-progress = Kadras { $done } / { $total }
anim-encoding = Rašomas GIF…
anim-export = Eksportuoti…

## ======== Išklotinė ========

unfold-paper-fit = Pritaikyti prie dalių
unfold-paper-a4 = A4
unfold-paper-a3 = A3
unfold-paper-a2 = A2

## Išklotinės langas ir eksportas

unfold-title = Išklotinė
unfold-selection = Pasirinkimas
unfold-everything = Viskas kontekste
unfold-nothing = Išklotinė: nėra ką skleisti (pasirinkite paviršius ar grupes)
# $name is the document name, $scale like "1:10".
unfold-footer = { $name } - išklotinė { $scale }
# $n is the sheet number out of $total.
unfold-footer-sheet = { $name } - išklotinė { $scale } - lapas { $n } iš { $total }
# $oversize is the number of pieces larger than the sheet (0 = none).
unfold-done = Išskleista paviršių: { $faces }, dalių: { $pieces }, lapų: { $sheets }{ $oversize ->
        [0] {""}
       *[other] {" "}(didesnių už lapą: { $oversize })
    }: { $path }
unfold-export-failed = Eksportas nepavyko: { $error }
# $what is "Selection" or "Everything in the context".
unfold-no-faces = { $what }: nėra paviršių. Pasirinkite paviršius, grupes ar komponentus.
unfold-summary = { $what }: paviršių { $faces } → dalių { $pieces }, lapų { $sheets }
unfold-oversize = Dalių, didesnių už lapą: { $n }. Pasirinkite didesnį popierių ar mastelį.
unfold-scale = Mastelis 1:
unfold-paper = Popierius
unfold-landscape = Gulsčias
unfold-glue-tabs = Klijavimo skiltelės
unfold-edge-numbers = Briaunų numeriai
unfold-soft-folds = Lenkimai ant lygių briaunų
unfold-sheet-of = Lapas { $n } iš { $total }
unfold-legend-cut = ━ pjūvis
unfold-legend-valley = ╍ slėnis
unfold-legend-mountain = ━·━ kalnas
unfold-export = Eksportuoti:

## ======== Šešėliuoklių redaktorius ========

## Įtrauktieji šablonai
shader-preset-wood = Mediena
shader-preset-water = Vanduo
shader-preset-reeded-glass = Rievėtas stiklas
shader-preset-glow = Švytėjimas

## Užbaigimo ir iškvietimo užuominos (pagalbinių funkcijų, WGSL integruotųjų funkcijų ir Surface laukų aprašai)
shader-fn-surface = šešėliuoklė: grąžina pakeistą paviršių
shader-fn-hash3 = pseudoatsitiktinė reikšmė 0..1 kiekvienam taškui
shader-fn-noise = glotnus reikšmių triukšmas 0..1
shader-fn-fbm = sluoksniuotas triukšmas 0..1 (debesys, marmuras, iškraipymas)
shader-fn-srgb = sRGB (0..1) pasirinkta spalva → tiesinė
shader-fn-hsv = atspalvis / sodrumas / reikšmė (0..1) → tiesinė spalva
shader-fn-tangent_of = vienetinis vektorius skersai n
shader-fn-bump = n, pakreiptas pagal aukščio lauko nuolydį
shader-fn-abs = absoliutinė reikšmė
shader-fn-min = mažesnioji
shader-fn-max = didesnioji
shader-fn-clamp = x apribotas nuo lo iki hi
shader-fn-saturate = x apribotas nuo 0 iki 1
shader-fn-mix = a + (b − a)·t
shader-fn-step = 0 žemiau ribos, 1 nuo jos
shader-fn-smoothstep = glotnus 0→1 tarp lo ir hi
shader-fn-fract = x − floor(x)
shader-fn-floor = apvalinti žemyn
shader-fn-ceil = apvalinti aukštyn
shader-fn-round = artimiausias sveikasis skaičius
shader-fn-sign = −1, 0 arba 1
shader-fn-sqrt = kvadratinė šaknis
shader-fn-pow = x laipsniu y
shader-fn-exp = e laipsniu x
shader-fn-log = natūrinis logaritmas
shader-fn-sin = sinusas (radianai)
shader-fn-cos = kosinusas (radianai)
shader-fn-tan = tangentas (radianai)
shader-fn-atan2 = taško (x, y) kampas, radianai
shader-fn-length = vektoriaus ilgis
shader-fn-distance = length(a − b)
shader-fn-normalize = vienetinis vektorius išilgai v
shader-fn-dot = skaliarinė sandauga
shader-fn-cross = dviejų vec3 vektorinė sandauga
shader-fn-reflect = i, atspindėtas per normalę n
shader-fn-select = t, kai cond, kitaip f
shader-fn-vec2 = 2 komponentų vektorius
shader-fn-vec3 = 3 komponentų vektorius
shader-fn-vec4 = 4 komponentų vektorius
shader-surface-base = tiesinė pagrindinė spalva
shader-surface-alpha = nepermatomumas
shader-surface-metallic = 0 = dielektrikas, 1 = metalas
shader-surface-roughness = 0 = veidrodis, 1 = matinis
shader-surface-transmission = stiklas: 1 = šviesa praeina
shader-surface-emission = skleidžiama šviesa, tiesinė, stiprumas įskaičiuotas
shader-surface-normal = pasaulio normalė, vienetinė, atsukta į žiūrovą
shader-surface-pos = modelio padėtis, mm (z = aukštyn)
shader-surface-uv = plokštuminės tekstūros koordinatės, mm
shader-surface-view = vienetinė kryptis į akį
shader-surface-time = sekundės
shader-param-tag = medžiagų redaktoriaus slankiklis, skaitomas kaip p.name (daugiausia 8)

## ======== Dialogai ========

## Automatinio įrašymo atkūrimas
dialog-untitled = Be pavadinimo
dialog-age-recently = neseniai
dialog-age-under-minute = prieš mažiau nei minutę
# $n is a number
dialog-age-minutes = { $n ->
        [one] prieš { $n } minutę
        [few] prieš { $n } minutes
        [many] prieš { $n } minutės
       *[other] prieš { $n } minučių
    }
dialog-age-hours = { $n ->
        [one] prieš { $n } valandą
        [few] prieš { $n } valandas
        [many] prieš { $n } valandos
       *[other] prieš { $n } valandų
    }
dialog-age-days = { $n ->
        [one] prieš { $n } dieną
        [few] prieš { $n } dienas
        [many] prieš { $n } dienos
       *[other] prieš { $n } dienų
    }

## Išsaugoti pakeitimus
dialog-save-title = Išsaugoti pakeitimus?
# $name is the document name
dialog-save-question = Ar norite išsaugoti pakeitimus, padarytus faile „{ $name }“?
dialog-save-warning = Jei neišsaugosite, pakeitimai bus prarasti.
dialog-dont-save = Neišsaugoti
dialog-save = Išsaugoti

## Atkurti automatiškai išsaugotą modelį
dialog-recover-title = Atkurti automatiškai išsaugotą modelį?
# $name is a file name; $age is dialog-age-* text ("5 minutes ago")
dialog-recover-copy = Varstotas rado automatiškai išsaugotą failo „{ $name }“ kopiją, sukurtą { $age }, kuri naujesnė už išsaugotą failą.
dialog-recover-untitled = Varstotas rado automatiškai išsaugotą modelį be pavadinimo, sukurtą { $age }.
dialog-recover-note = Joje gali būti neišsaugotų pakeitimų, pavyzdžiui, po sutrikimo.
dialog-discard = Atmesti
dialog-discard-hint = Ištrinti automatiškai išsaugotą kopiją
dialog-recover = Atkurti
dialog-not-now = Ne dabar
dialog-not-now-hint = Palikti automatiškai išsaugotą kopiją vėlesniam laikui

## Dydžio keitimas
dialog-resize-title = Keisti dydį?
dialog-resize-model = Ar norite pakeisti modelio dydį?
dialog-resize-group = Ar norite pakeisti aktyvios grupės ar komponento dydį?
# $factor is the scale factor, e.g. 1.2500
dialog-resize-scaled = Viskas jame keičiama { $factor } karto aplink jo pradžios tašką.
dialog-resize-keys = Enter = Taip · Esc = Ne
dialog-yes = Taip
dialog-no = Ne

## Reikšmės užklausa
dialog-invalid-value = Netinkama reikšmė

## Bendri užrašai
dialog-width = Plotis
dialog-height = Aukštis

## Eksportuoti 2D grafiką
dialog-export-title = Eksportuoti 2D grafiką
dialog-export-aspect = Naudoti rodinio kraštinių santykį
dialog-export-viewport = Rodinio dydis
# $w and $h are pixel counts
dialog-export-viewport-hint = { $w } × { $h } px (ekrano taškai)
dialog-export-double = Padvigubinti vaizdo dydį
dialog-export-transparent = Skaidrus fonas
dialog-export-axes = Ašys
dialog-export-guides = Pagalbinės linijos
dialog-export-dims = Matmenys
dialog-export-note = Pasirinkimas ir įrankių peržiūros niekada neeksportuojami.
dialog-export = Eksportuoti…
dialog-filter-png = PNG vaizdas

## Eksportuoti 2D brėžinį
dialog-drawing-title = Eksportuoti 2D brėžinį
dialog-drawing-intro = Dabartinis rodinys linijomis: SVG arba PDF.
dialog-drawing-scale = Mastelis 1:
dialog-drawing-perspective = Perspektyvinis rodinys: pritaikytas A4 lapui. Kamera → Lygiagreti projekcija, jei reikia mastelio.
dialog-drawing-fitted = Pritaikyta A4 lapui.
dialog-drawing-profiles = Kontūrai (storos linijos)
dialog-drawing-hidden = Paslėptos linijos (punktyrinės)
dialog-drawing-dims = Matmenys ir tekstas
dialog-drawing-sections = Pjūvis
dialog-drawing-pdf = PDF…
dialog-drawing-svg = SVG…

## Įdėti 3D tekstą
dialog-text3d-title = Įdėti 3D tekstą
dialog-text3d-font = Šriftas
dialog-text3d-filled = Užpildytas
dialog-text3d-extruded = Išstumtas
dialog-text3d-hint = Enter pradeda naują eilutę, %CMD%+Enter įdeda tekstą.
dialog-text3d-place = Įdėti

## Šablono pasirinkimas
dialog-template-title = Pasirinkite šabloną
dialog-template-intro = Nauji modeliai pradedami su šiais vienetais. Numatytąjį galite pakeisti Nustatymuose.

## Apie
dialog-about-tagline = Lengvas, vietinis 3D modeliavimo įrankis braižymui ir kūrimui.
# $version is the app version
dialog-about-version = Versija { $version }
app-start-failed-title = Nepavyko paleisti Varstotas
app-start-failed = Varstotas nepavyko atverti lango:

    { $error }

    3D vaizdui reikia Vulkan, DirectX 12, Metal arba OpenGL 3.3. Atnaujinkite vaizdo plokštės tvarkyklę; virtualioje mašinoje įjunkite 3D spartinimą. Norėdami išbandyti kitą grafikos API, nustatykite aplinkos kintamąjį WGPU_BACKEND = gl, vulkan arba dx12.
dialog-about-build = Kompiliacija { $build } · { $commit }
dialog-about-dev-build = Kūrėjo kompiliacija · { $commit }
dialog-about-licence = Laisvoji programinė įranga pagal MIT licenciją.
dialog-about-licences = Licencijos…

## ======== Lapai ========

sheets-title = Lapai
sheets-custom = Pasirinktinis
sheets-none-yet = Lapų dar nėra.
sheets-none-first = Nėra lapo: pirmiausia pridėkite (Langas → Lapai)
sheets-nothing-to-export = Lapai: nėra ką eksportuoti (pirmiausia pridėkite lapą)
# $i is the sheet's number, $n the number of sheets
sheets-exported-one = Eksportuotas lapas { $i } iš { $n }: { $path }
sheets-exported-all = Eksportuota lapų: { $n }: { $path }
sheets-export-failed = Eksportas nepavyko: { $error }
sheets-add-first = Norėdami pradėti, pridėkite lapą

## Atšaukimo žingsniai
sheets-undo-add = Pridėti lapą
sheets-undo-edit = Redaguoti lapą
sheets-undo-duplicate = Dubliuoti lapą
sheets-undo-delete = Ištrinti lapą

## Lapų sąrašas ir savybės
sheets-add = + Lapas
sheets-duplicate = Dubliuoti
sheets-name = Pavadinimas
sheets-paper = Popierius
sheets-landscape = Gulsčias
sheets-title-block = Antraštės blokas
sheets-tb-title = Pavadinimas
sheets-tb-author = Autorius
sheets-tb-date = Data

## Elementų pridėjimas
sheets-add-viewport = + Rodinys
sheets-add-viewport-hint = Dabartinis modelio rodinys
sheets-add-text = + Tekstas
sheets-add-credits = + Autorystė
sheets-add-credits-hint = Atsisiųstų modelių autorystės nuorodos (pavadinimas, autorius, svetainė, licencija): tekstas su {"{credits}"}, užpildomas braižant
sheets-remove = Pašalinti
sheets-add-views = + Standartiniai rodiniai
sheets-add-views-hint = Priekis, viršus ir kairė standartiniu masteliu su bendraisiais matmenimis bei izometrinis rodinys
sheets-add-table = + Dalių lentelė
sheets-add-table-hint = Grupės ir komponentai su dydžiais

## Pasirinktas elementas
# $n is the item's number
sheets-viewport-n = Rodinys { $n }
sheets-text-n = Tekstas { $n }
sheets-table-n = Dalių lentelė { $n }
sheets-shows = Rodo
sheets-captured = Užfiksuotas rodinys
sheets-update-view = Atnaujinti pagal dabartinį rodinį
sheets-scale = Mastelis
sheets-scale-hint = Mastelis reikalauja Lygiagrečios projekcijos
sheets-position = Padėtis
sheets-size = Dydis
sheets-height = Aukštis
sheets-text-height = Teksto aukštis
sheets-profiles = Kontūrai
sheets-hidden = Paslėptos linijos
sheets-dims = Matmenys
sheets-sections = Pjūviai
sheets-frame = Rėmelis
sheets-caption = Antraštė
sheets-overall = Bendrieji matmenys
sheets-hint = Spustelėkite elementą peržiūroje, kad jį redaguotumėte; vilkite, kad perkeltumėte, vilkite kampą, kad pakeistumėte dydį.

## Apačia
# $paper is a paper name; the others are counts
sheets-summary = { $paper } — rodinių: { $viewports }, pagal mastelį: { $scaled }, tekstų: { $texts }
sheets-svg-hint = Šis lapas
sheets-pdf-hint = Visi lapai, po puslapį kiekvienam
sheets-export = Eksportuoti:

## ======== Komponento parametrai ========

rig-title = Komponento parametrai
rig-select-hint = Pasirinkite komponentą (ne grupę) ir pasirinkite Langas → Komponento parametrai arba Redaguoti parametrus… Savybėse.
rig-state-has = — su parametrais
rig-state-none = — parametrų dar nėra: Taikyti juos prideda
rig-no-parts = Komponente nėra judinamų dalių: sugrupuokite jo detales viduje (durys, stalčius…) — atidarykite jį ir naudokite Sukurti grupę.

## Savybių blokas
# $name is the component's name; $n the number of copies
rig-info-copies = { $name } — kopijų: { $n }
rig-no-params = Parametrų dar nėra
rig-reset = Atkurti
rig-reset-hint = Grįžti prie numatytųjų reikšmių
rig-edit = Redaguoti parametrus…
rig-edit-hint = Langas → Komponento parametrai: parametrai, formulės, dalių judesiai, scenarijus
# $n is the number of motions
rig-info-summary = Komponentas su parametrais (judesių: { $n })

## Atšaukimo žingsniai ir pranešimai
rig-undo-reset = Atkurti parametrus
rig-undo-edit = Redaguoti parametrus
rig-undo-add = Pridėti parametrus
rig-undo-remove = Pašalinti parametrus
rig-select-rigged = Pasirinkite komponentus su parametrais
rig-select-component = Pasirinkite komponentą, kad suteiktumėte jam parametrų
rig-err-group = Grupės negali turėti parametrų: pirmiausia paverskite ją komponentu (Redaguoti → Sukurti komponentą)
rig-err-generated = Šį komponentą sukuria generatorius arba parametrinė grupė
rig-applied = Parametrai pritaikyti
rig-removed = Parametrai pašalinti
rig-err-place-copy = Įdėkite komponento kopiją, kad galėtumėte pasirinkti modelyje
rig-pick-axis = Spustelėkite du sukimosi ašies taškus
rig-pick-slide = Spustelėkite du taškus: slydimo kryptis
rig-err-same-points = Abu taškai sutampa

## Parametrų lentelė
rig-parameters = Parametrai
rig-parameters-hint = Reikšmės, kurias turi kiekviena kopija (jas rodo Savybės)
rig-col-name = Pavadinimas
rig-col-label = Etiketė
rig-col-type = Tipas
rig-col-min = Min.
rig-col-max = Maks.
rig-col-step = Žingsnis
rig-col-default = Numatytoji
rig-col-options = Parinktys
rig-as-drawn = (kaip nubraižyta)
rig-remove-param = Pašalinti parametrą
rig-add-param = + Parametras

## Formulės
rig-formulas = Formulės
rig-formulas-hint = Iš parametrų apskaičiuotos reikšmės (Rhai reiškiniai), kurias gali naudoti judesiai ir scenarijus
rig-add-formula = + Formulė

## Judesiai
rig-motions = Judesiai
rig-motions-hint = Dalis (tiesiogiai viduje esanti grupė / komponentas), kurią parametras sukioja, slenka, rodo, dažo arba kuriai suteikia parametro reikšmę (Nustatyti parametrą: dalis, kuri pati turi parametrų — sugeneruotas komponentas, pvz., Braižyti → Figūros cilindras, arba komponentas su parametrais). Sukimas / slinkimas: parametro diapazonas atvaizduojamas į Nuo..Iki (laipsniai / ilgis); be diapazono arba su reiškiniu reikšmė pati yra judesys
rig-motion-rotate = Sukti
rig-motion-slide = Slinkti
rig-motion-show = Rodyti / slėpti
rig-motion-paint = Dažyti
rig-motion-set = Nustatyti parametrą
rig-parameter = (parametras)
rig-part-no-params = Dalis neturi parametrų
rig-part-param-hint = Pačios dalies parametras, nustatytas į reikšmę
rig-by = pagal
rig-expression = (reiškinys)
rig-from = nuo
rig-to = iki
rig-remove-motion = Pašalinti judesį
rig-axis-through = Ašis per
rig-along = išilgai
rig-direction = Kryptis
rig-pick = Pasirinkti…
rig-pick-hint = Spustelėkite du modelio taškus
rig-axes-note = mm, komponento ašys
rig-add-motion = + Judesys

## Scenarijus
rig-script = Scenarijus
rig-script-hint = Neprivalomas Rhai kodas, vykdomas po judesių komponento viduje: fn update(p) {"{"} … {"}"} su visa scenarijų API (rotate, move, paint, show / hide, set_params, box…); child(name) / child(name, true) randa dalį pagal pavadinimą (giliai), part.min / max / size / bounds — jos ribos; p — reikšmės ir formulės
rig-add-script = + Scenarijus

## Mygtukai
rig-add = Pridėti parametrus
rig-revert = Grąžinti
rig-remove = Pašalinti parametrus
rig-remove-hint = Kopijos vėl rodo komponentą, kaip nubraižyta

## ======== Generatoriai ========

## Generatoriaus redaktorius
gen-editor-title = Generatoriaus redaktorius
gen-new = Naujas
gen-new-hint = Pradėti nuo šablono
gen-open = Atidaryti…
gen-filter = Generatorius
gen-save = Išsaugoti bibliotekoje
gen-save-hint = Įrašyti generatorių į { DATA_DIR() }/generators (jis rodomas Komponentų dėkle)
gen-apply-hint = Atnaujinti šį dokumento generatorių: visi jo komponentai perkuriami (vienas atšaukimo žingsnis)
gen-apply-doc = Taikyti dokumentui
gen-apply-doc-hint = Naudoti šį šaltinį dokumente: jo komponentai perkuriami (vienas atšaukimo žingsnis)
gen-place = Įdėti
gen-place-hint = Įdėti komponentą su peržiūros reikšmėmis
# $name is the generator's name in the document
gen-in-model = Modelyje: { $name }
gen-not-saved = neišsaugota
gen-build-failed = Sukurti nepavyko
gen-drag-turn = Vilkite, kad pasuktumėte
gen-defaults = Numatytieji
gen-no-home = Nėra namų aplanko
# $path is a file path; $error the system's message
gen-saved = Išsaugota { $path }
gen-cannot-save = Nepavyko išsaugoti { $path }: { $error }
gen-name-taken = Dokumente jau yra generatorius pavadinimu { $name }
gen-undo-update = Atnaujinti generatorių { $name }
# $old and $name are generator names
gen-renamed-applied = { $old } → { $name }: pritaikyta dokumentui
gen-same-source = { $name }: dokumentas jau naudoja šį šaltinį
gen-applied = { $name }: pritaikyta dokumentui

## Generatorių sąrašas ir plytelės
# $label is a parameter label
gen-not-length = { $label }: ne ilgis
gen-formulas = Formulės
gen-origin-bundled = Įtrauktas pavyzdys (Redaguoti → Išsaugoti bibliotekoje sukuria jūsų kopiją)
gen-origin-document = Šiame dokumente
gen-tile-error = Klaida: { $error }
# $note is the description and origin lines
gen-tile-tip =
    Generatorius — parametrai Savybėse
    { $note }
gen-edit = Redaguoti generatorių…
gen-show-in-finder = Rodyti failų tvarkyklėje
    .macos = Rodyti Finder
    .windows = Rodyti Failų naršyklėje
gen-delete = Ištrinti…
gen-error = Generatorius: { $error }

## Savybių blokas
# $name is the generator's name; $n the number of components
gen-info-components = { $name } — komponentų: { $n }
gen-info-generator = Generatorius: { $name }
gen-reset-hint = Grįžti prie generatoriaus numatytųjų reikšmių
gen-unique-generated = Jo generatoriaus kopija šiam komponentui („Lentyna 2“ skiltyje Modelyje): redaguojant tą scenarijų keičiasi tik ji
gen-unique-hint = Suteikti šiam komponentui savo apibrėžtį: ją redaguojant kitos kopijos nebekeičiamos
gen-unique-already = Ši kopija jau unikali: jos apibrėžties ar generatoriaus niekas kitas nenaudoja
gen-unique-select = Pasirinkite vieną komponentą
gen-make-unique = Padaryti unikalų
gen-mixed = Pasirinkti komponentai skiriasi; pakeitimas nustato juos visus
gen-select-generated = Pasirinkite sugeneruotus komponentus

## ======== Dokumento informacija ========

model-info-title = Dokumento informacija
model-info-scene = Scenoje
model-info-scene-hint = Kas nubraižyta: skaičiuojamas kiekvienos įdėtos kopijos turinys
model-info-file = Faile
model-info-file-hint = Ką saugo dokumentas: kiekviena apibrėžtis vieną kartą, naudojama ar ne
model-info-libraries = Bibliotekos
# $n is a number, already formatted
model-info-files = Failų: { $n }
# $used, $all and $unused are formatted numbers
model-info-used-of = { $used } iš { $all } (nenaudojama: { $unused })

## Eilutės
model-info-row-polygons = Daugiakampiai (paviršiai)
model-info-row-triangles = Trikampiai
model-info-row-edges = Briaunos
model-info-row-groups-placed = Įdėtos grupės
model-info-row-components-placed = Įdėti komponentai
model-info-row-components = Komponentai
model-info-row-groups = Grupės
model-info-row-materials = Medžiagos
model-info-row-textures = Tekstūros
model-info-row-tags = Žymės
model-info-row-scenes = Scenos
model-info-row-sheets = Maketo lapai
model-info-row-generators = Generatoriai
model-info-row-lights = Šviesos
model-info-row-guides = Pagalbinės linijos / matmenys
model-info-row-file-size = Failo dydis (išsaugoto)
model-info-row-lib-components = Komponentų biblioteka
model-info-row-lib-materials = Medžiagų biblioteka
model-info-preview-image = Peržiūros vaizdas
model-info-none = nėra
model-info-show = Rodyti…
model-info-show-hint = Failas → Peržiūros vaizdas → Rodyti…

## Peržiūros vaizdo langas
model-info-preview-title = Peržiūros vaizdas
model-info-no-preview = Nėra peržiūros vaizdo — Komponentų biblioteka nupiešia modelį
model-info-preview-note = Išsaugomas su failu; Komponentų biblioteka rodo jį šiam failui.
model-info-from-file = Iš vaizdo failo…
model-info-from-view = Iš dabartinio rodinio

## Išvalyti nenaudojamus
model-info-purge = Išvalyti nenaudojamus
model-info-purge-note = Pašalina tai, ko scenoje niekas nenaudoja (Redaguoti → Atšaukti grąžina).
# $n is the number of unused definitions / materials
model-info-purge-defs = Komponentai ir grupės ({ $n })
model-info-purge-mats = Medžiagos ({ $n })
model-info-nothing-to-purge = Nėra nenaudojamų elementų valymui
model-info-purge-failed = Išvalyti nepavyko
# $n is a count
model-info-purged-components = komponentų: { $n }
model-info-purged-groups = grupių: { $n }
model-info-purged-materials = medžiagų: { $n }
# $parts is a comma-separated list of the three texts above
model-info-purged = Išvalyta: { $parts }

## ======== Papildiniai ========

## Papildinio antraštės klaidos (rodomos papildinių autoriams)
# $label is a parameter label; $value is the text entered
ext-param-invalid = { $label }: netinkama reikšmė: { $value }
# $text is the @param line
ext-param-needs-default = @param reikalauja `name = default`: { $text }
ext-param-bad-name = netinkamas parametro pavadinimas `{ $key }`
ext-param-unclosed-quote = neuždarytos kabutės: { $text }
ext-param-unknown-type = nežinomas parametro tipas `{ $ty }` (length, int, number, text, bool)
ext-pick-needs-number = @pick reikalauja taškų skaičiaus: { $text }
ext-pick-range = @pick: nuo 1 iki 16 taškų
ext-pick-default = Spustelėkite taškų: { $n }
ext-select-default = Pasirinkite objektus
ext-empty-name = tuščias @name

## Medžiagų papildiniai
ext-mat-plugin-failed = papildinys nepavyko
ext-mat-no-answer = papildinys neišvedė atsakymo
# $error is the JSON parser's message; $answer is the start of the plugin's answer
ext-mat-not-json = jo atsakymas nėra JSON ({ $error }): { $answer }
ext-mat-catalogue-array = catalogue() turi grąžinti medžiagų masyvą
ext-mat-no-home = nėra namų aplanko
ext-mat-default-name = Medžiaga
ext-mat-import-nothing = import_material() nieko negrąžino
ext-tex-bad-size = Įterptas vaizdas: "size" turi būti viena kraštinė arba [plotis, aukštis] nuo 1 iki 8192 pikselių
ext-tex-no-pixels = Įterptas vaizdas: pikselius pateikite "rgb", "rgba" arba "grey" (masyvas 0..255, base64 arba blob)
ext-tex-pixel-count = Įterptas vaizdas: tikėtasi baitų: { $want }, pateikta: { $got }
ext-mat-spec-object = medžiagos specifikacija turi būti objektas

## Papildinių meniu ir dialogai
ext-script-undo = Scenarijus
# $error is the plugin header parser's message
ext-header-error = antraštė: { $error }
ext-no-plugins = Nėra papildinių
# $n is the number of plugins
ext-failed-to-load = Nepavyko įkelti papildinių: { $n }…
ext-reload = Įkelti papildinius iš naujo
ext-open-folder = Atidaryti papildinių aplanką
ext-script-gone = scenarijaus nebėra
# $name is the plugin's or script's name (log lines)
ext-run-script = scenarijus { $name }
ext-run-plugin = papildinys { $name }
ext-plugin-failed = papildinys nepavyko
ext-done = { $name } baigta
ext-exited = { $name } baigėsi su klaida ({ $status })
ext-python-no-bridge = Python scenarijams reikia tilto (išjungtas per VARSTOTAS_NO_BRIDGE)
ext-python-busy = Python scenarijus jau vykdomas
ext-python-cant-start = Nepavyko paleisti { $python }: { $error } (nustatykite VARSTOTAS_PYTHON)
ext-pick-points = Pasirinkite taškus
ext-select-objects = Pasirinkite objektus
ext-run = Vykdyti

## ======== Scenarijų konsolė ir scenarijų žinynas ========

## Scenarijų konsolė
console-title = Scenarijų konsolė
console-script-failed = scenarijus nepavyko
# $error is the script's error text
console-script-error = Scenarijaus klaida: { $error }
# $name is the script file name; this is the undo step name
console-script-undo = Scenarijus { $name }
# $path is a file path; $error is the system's message
console-cannot-read = Nepavyko perskaityti { $path }: { $error }
console-hint = Rhai scenarijai modeliui. Išbandykite: box(ORIGIN, 1000, 600, 18)  — %CMD%+Enter paleidžia
console-run = Paleisti
console-run-hint = %CMD%+Enter
console-open = Atverti…
console-save = Išsaugoti…
console-stop-python = Stabdyti Python
console-clear = Išvalyti išvestį
# $port is a TCP port number
console-bridge = Python / MCP tiltas: 127.0.0.1:{ $port }
console-bridge-hint = import varstotas; m = varstotas.connect()  — žr. { DATA_DIR() }/python
console-filter-rhai = Rhai scenarijus

## Scenarijų žinyno langas
console-ref-title = Scenarijų žinynas
console-search = Ieškoti
console-search-hint = funkcija, savybė ar žodis
console-fulltext = Pilnas tekstas: docs/SCRIPTING.md, docs/GENERATORS.md
console-copy-example = Kopijuoti pavyzdį
console-click-copy = Spustelėkite, kad nukopijuotumėte
console-topic-basics = Pagrindai
console-topic-console = Konsolė ir papildiniai
console-topic-generators = Generatoriai
console-topic-component = Komponento scenarijus
console-topic-functions = Funkcijos
console-topic-properties = Savybės
console-param-types = Parametrų tipai
console-generators-note = Viduje `build` veikia visa API (žr. Funkcijos). Scenarijus vykdomas tuščiame modelyje, kuriame yra dokumento medžiagos ir žymės; viskas, ką jis sukuria, tampa komponento turiniu.
console-functions-intro = Visos API funkcijos pagal skyrius. Spustelėkite parašą, kad jį nukopijuotumėte. `x` yra objektas, objektų masyvas arba antrinės grupės pavadinimas.
console-properties-intro = Po taško: `g.name`, `f.normal.x`, `door.size.z`, `[1, 2].len()`. Priskiriamas savybes galima nustatyti: `g.name = "Crate"`, `g.hidden = true`.
console-kind-objects = Objektai (paviršiai, briaunos, grupės, komponentai)
console-kind-vectors = Vektoriai
console-kind-bounds = Ribos — bounds(x), x.bounds
console-kind-arrays = Masyvai
console-kind-strings = Eilutės
console-nothing-found = Nieko nerasta
console-members = Savybės ir metodai
console-tags = Antraštės žymės
console-reference = ? Žinynas
console-reference-hint = Pagalba → Scenarijų žinynas: funkcijos, savybės, pavyzdžiai (F1 redaktoriuje: žodis prie žymeklio)

## Puslapis: pagrindai
console-ref-basics-1 = Scenarijai rašomi Rhai kalba — nedidele kalba su Rust / JavaScript panašia sintakse. Koordinatės yra pasaulio milimetrais; ilgiai su vienetais yra eilutės: `length("1.2m")`.
console-ref-basics-2 = Kalba
console-ref-basics-4 = Eilutės: `"text"`, jungiamos su `+`: `"w = " + w`. Eilučių metodai `trim()`, `to_upper()` pakeičia eilutę vietoje ir nieko negrąžina: `s.trim(); if s == "" {"{"} … {"}"}`.
console-ref-basics-5 = Konstantos: `ORIGIN`, `X`, `Y`, `Z` (vienetinės ašys). Vektorius galima sudėti, atimti ir keisti mastelį: `ORIGIN + X * 100`; `.x .y .z`, `length()`, `normalize()`, `dot(b)`, `cross(b)`, `distance(b)`.
console-ref-basics-6 = Objektai
console-ref-basics-7 = Paviršiai, briaunos, grupės ir komponentai yra nuorodos: `let g = box(ORIGIN, 100, 100, 100); g.name = "Crate";`. Po operacijų, kurios sujungia ar perpjauna geometriją, sena nuoroda gali tapti negaliojanti — patikrinkite `e.valid`.
console-ref-basics-8 = Kur tikimasi objekto, eilutė įvardija dabartinio konteksto grupę / komponentą: `move("Door", vec3(0, 10, 0))`. `child("Knob", true)` ieško ir įdėtose grupėse.
console-ref-basics-9 = Kiekvienas objektas turi `kind`, `bounds`, `size`, `min`, `max`, `center`, `hidden`; grupės turi ir `name`, `definition`, `material`, `tag` (žr. Savybės).
console-ref-basics-10 = Kontekstas ir atšaukimas
console-ref-basics-11 = Nauja geometrija patenka į dabartinį kontekstą: grupę, atvertą scenarijui prasidėjus, arba tą, į kurią įeita su `enter(g)` (išeinama su `exit()`).
console-ref-basics-12 = Vienas paleidimas yra vienas atšaukimo žingsnis. Scenarijus, kuris nepavyksta, nieko nekeičia; rodoma klaida su eilute — ji išsaugoma ir Langas → Žurnalas.
console-ref-basics-13 = `print(x)` rašo į išvestį; paskutinės išraiškos reikšmė taip pat išspausdinama. Scenarijai sustabdomi po 30 s (generatoriai po 10 s).
## Puslapis: konsolė
console-ref-console-1 = Langas → Scenarijų konsolė vykdo kodą atvertame modelyje. %CMD%+Enter paleidžia, %CMD%+↑ / ↓ naršo ankstesnius paleidimus, Atverti… / Išsaugoti… dirba su `.rhai` failais. Failas → Paleisti scenarijų… paleidžia failą.
console-ref-console-2 = Užbaigimas renkant rodo API funkcijas, po taško — savybes (pagal kairėje esantį tipą), o virš žymeklio — šaukiamos funkcijos parašą. F1 atveria šį žinyną ties žodžiu prie žymeklio.
console-ref-console-3 = Antraštė: dialogai ir meniu elementai
console-ref-console-4 = Eilutės `// @tag …` viršuje paverčia scenarijų papildiniu: Paleisti pirmiausia paklausia parametrų, taškų ir objektų. Failas aplanke `{ DATA_DIR() }/plugins` tampa Papildiniai meniu elementu.
console-ref-console-6 = Žymės
## Puslapis: generatoriai
console-ref-generators-1 = Generatorius yra komponentas su parametrais: scenarijus, kurio `build(p)` sukuria komponento turinį jo paties ašyse (pradžios taškas yra įterpimo taškas). Kiekviena kopija turi savo reikšmes Savybėse; kopijos su vienodomis reikšmėmis dalijasi vienu apibrėžimu.
console-ref-generators-2 = Langas → Generatorių redaktorius (arba Redaguoti generatorių… ant plytelės): kairėje šaltinis, dešinėje gyvas peržiūros langas su parametrų laukais; Taikyti atnaujina dokumentą, Išsaugoti bibliotekoje įrašo failą.
console-ref-generators-4 = Antraštė
console-ref-generators-5 = privaloma; generatoriaus raktas dokumente
console-ref-generators-6 = `p` reikšmė; kiekviena intervalo dalis neprivaloma
console-ref-generators-7 = iš parametrų apskaičiuota reikšmė (Savybėse tik skaitoma)
console-ref-generators-8 = reikšmės, kurioms ji klaidinga, atmetamos
console-ref-generators-9 = Parametrų tipai
## Puslapis: komponentas
console-ref-component-1 = Langas → Komponento parametrai suteikia bet kuriam komponentui parametrus: reikšmių eilutes, formules, jo dalių judesius (sukti, stumti, rodyti, dažyti, nustatyti dalies parametrą) ir pasirinktinį scenarijų.
console-ref-component-2 = Scenarijus apibrėžia `update(p)`. Jis vykdomas po judesių, komponento viduje, su visa scenarijų API — tomis pačiomis funkcijomis kaip konsolėje ir generatoriuose. `p` laiko reikšmes ir formules.
console-ref-component-4 = Dalių radimas
console-ref-component-5 = grupė / komponentas tiesiai komponento viduje
console-ref-component-6 = ieškoma ir įdėtose grupėse (laimi artimiausia)
console-ref-component-7 = dalies viduje
console-ref-component-8 = visos dalys (ir įdėtosios)
console-ref-component-9 = dalies dabartinės ribos (pasaulio)
console-ref-component-10 = paslėpti, kaip hide(part)
console-ref-component-11 = Kaip tai veikia
console-ref-component-12 = Scenarijus keičia kopiją (šių reikšmių variantą), niekada ne pagrindinį: jo pakeistos dalys nukopijuojamos į variantą, nepaliestos lieka bendros. Kopijos su vienodomis reikšmėmis dalijasi vienu variantu.
console-ref-component-13 = Pakeitimai, atlikti kopijos viduje (dukart spustelėjus), grįžta į pagrindinį be to, ką padarė scenarijus: tai, ką jis pridėjo, atmetama, perkėlimai / slėpimas / dažymas atšaukiami.
console-ref-component-14 = Klaidos rodomos paspaudus Taikyti ir išsaugomos Langas → Žurnalas.

## Kodo redaktorius
# $sel is the selected completion's number, $n the number of completions
console-completion-count = { $sel } iš { $n }

## ======== Žurnalas ========

log-level-error = klaida
log-level-warning = įspėjimas
log-title = Žurnalas
# $n is a count
log-errors = Klaidos { $n }
log-warnings = Įspėjimai { $n }
log-filter = filtras
log-clear = Išvalyti
log-copy = Kopijuoti
log-copy-hint = Kopijuoti rodomus įrašus
log-empty = Šiame seanse klaidų ar įspėjimų nėra.
log-no-match = Nieko neatitinka filtrų.
# $n is the number of errors not seen yet
log-badge-errors = ● { $n ->
        [one] { $n } klaida
        [few] { $n } klaidos
       *[other] { $n } klaidų
    }
log-badge = Žurnalas
log-badge-hint = Langas → Žurnalas: klaidos ir įspėjimai iš visur (scenarijai, importas / eksportas, bibliotekos …)

## ======== Licencijos ========

licences-tab-varstotas = Varstotas
licences-tab-components = Atvirasis kodas
licences-tab-formats = Failų formatai
licences-tab-libraries = Internetinės bibliotekos

licence-cc0 = CC0
licence-cc-by = CC-BY
licence-cc-by-sa = CC-BY-SA
licence-cc-by-nc = CC-BY-NC
licence-cc-by-nd = CC-BY-ND
licence-standard = Standartinė / asmeniniam naudojimui
licence-unknown = Nežinoma

licence-cc-by-nc-note = nekomercinė: neparduokite to, ką su ja sukūrėte
licence-cc-by-nd-note = be išvestinių darbų: nesidalykite pakeista kopija
licence-cc-by-sa-note = dalytis tomis pačiomis sąlygomis: pakeistas kopijas platinkite pagal tą pačią licenciją
licence-standard-note = paties tinklalapio licencija (nemokama / asmeniniam naudojimui / naudojimo sąlygos): dažnai be komercinio naudojimo arba be paties modelio failo platinimo — patikrinkite
licence-unknown-note = licencija nežinoma: tik savo reikmėms

# Licences window: labels and the explanations of formats and online libraries (the licence texts themselves are not translated).

## Langas
licences-title = Licencijos
licences-count = Komponentų: { $n }
licences-filter = Filtras: pavadinimas arba licencija
licences-save = Išsaugoti kaip tekstą…
licences-save-hint = Visi komponentai ir jų licencijų tekstai viename faile
licences-saved = Išsaugota { $path }
licences-pick = Pasirinkite komponentą, kad pamatytumėte jo licenciją.
licences-all-text-title = Varstotas naudoja šiuos atvirojo kodo komponentus.
licences-applies-to = Taikoma: { $users }

## Varstotas skirtukas
licences-own = Varstotas yra laisva programinė įranga pagal MIT licenciją: galite ją naudoti asmeniniam ir komerciniam darbui, kopijuoti, keisti ir platinti.
licences-programs = Programos, naudojamos, kai jas turite
licences-yours = Jūsų modeliai priklauso jums. Iš internetinių bibliotekų atsisiųsti modeliai ir medžiagos išlaiko savo licencijas (Internetinės bibliotekos; Failas → Eksportuoti → Priskyrimai).

## Programos


licence-program-oidn-use = vaizdo kūrimo DI triukšmo šalinimas
licence-program-oidn-licence = Apache-2.0 — nepristatoma, įkeliama iš jūsų diegimo

licence-program-python-use = Python papildiniai ir scenarijai
licence-program-python-licence = PSF licencija — nepristatoma, jūsų diegimas

## Failų formatų skirtukas
licence-formats-col-format = Formatas
licence-formats-col-varstotas = Varstotas
licence-formats-col-how = Kaip
licence-formats-col-note = Formatas

licence-format-varstotas-what = atverti, išsaugoti
licence-format-varstotas-how = Varstotas nuosavas formatas
licence-format-varstotas-note = MIT, kaip ir Varstotas

licence-format-gltf-what = importas, eksportas
licence-format-gltf-how = nuosavas skaitytuvas / rašytuvas; Draco tinkleliai per draco-core (Apache-2.0)
licence-format-gltf-note = Khronos atvirasis standartas, be autorinių atlyginimų

licence-format-obj-what = importas, eksportas
licence-format-obj-how = nuosavas skaitytuvas / rašytuvas
licence-format-obj-note = atvira, vieša specifikacija

licence-format-stl-what = importas, eksportas
licence-format-stl-how = nuosavas skaitytuvas / rašytuvas
licence-format-stl-note = atvira, vieša specifikacija

licence-format-ply-what = importas, eksportas
licence-format-ply-how = nuosavas skaitytuvas / rašytuvas
licence-format-ply-note = atvira, vieša specifikacija

licence-format-3mf-what = importas, eksportas
licence-format-3mf-how = nuosavas skaitytuvas / rašytuvas (zip + XML)
licence-format-3mf-note = 3MF Consortium specifikacija, be autorinių atlyginimų

licence-format-collada-what = importas, eksportas
licence-format-collada-how = nuosavas skaitytuvas / rašytuvas (vaizdai kaip failai šalia)
licence-format-collada-note = Khronos atvirasis standartas, be autorinių atlyginimų

licence-format-fbx-what = importas
licence-format-fbx-how = ufbx (MIT / Unlicense), nepriklausomas skaitytuvas
licence-format-fbx-note = Autodesk formatas; FBX yra Autodesk prekės ženklas, Varstotas nėra su juo susijęs

licence-format-dxf-what = importas, eksportas
licence-format-dxf-how = nuosavas skaitytuvas / rašytuvas
licence-format-dxf-note = Autodesk viešai paskelbtas formatas; DXF yra Autodesk prekės ženklas

licence-format-svg-pdf-what = eksportas (brėžiniai, išklotinės, lapai)
licence-format-svg-pdf-how = nuosavi rašytuvai
licence-format-svg-pdf-note = W3C / ISO atvirieji standartai

licence-format-images-what = tekstūros, vaizdai, aplinkos
licence-format-images-how = image crate (MIT / Apache-2.0)
licence-format-images-note = atvirieji formatai

licence-format-openexr-what = vaizdo kūrimo eksportas
licence-format-openexr-how = nuosavas rašytuvas
licence-format-openexr-note = Academy Software Foundation, BSD-3-Clause

licence-format-csv-json-what = taškų importas, paviršių eksportas (papildiniai)
licence-format-csv-json-how = nuosavas kodas
licence-format-csv-json-note = atvirieji formatai

licence-format-gif-what = animacijos eksportas
licence-format-gif-how = image crate (MIT / Apache-2.0)
licence-format-gif-note = atvirasis formatas

## Internetinių bibliotekų skirtukas
licence-libs-intro = Kiekvienas atsisiųstas modelis išlaiko savo šaltinį, autorių ir licenciją; Varstotas įspėja, kai padėto ar eksportuojamo modelio licencija riboja jo naudojimą (nekomercinė, be išvestinių darbų, dalytis tomis pačiomis sąlygomis, paties tinklalapio licencija, nežinoma).
licence-libs-col-library = Biblioteka
licence-libs-col-assets = Ištekliai
licence-libs-col-licence = Licencija
licence-libs-col-terms = Sąlygos

licence-lib-poly-haven-what = modeliai, medžiagos
licence-lib-poly-haven-licence = CC0
licence-lib-poly-haven-terms = Nemokama viešoji API, taip pat komercinėms programoms; prašoma rodyti „Powered by Poly Haven“ (rodoma virš jos išteklių) ir unikalų User-Agent (Varstotas).

licence-lib-ambientcg-what = medžiagos
licence-lib-ambientcg-licence = CC0
licence-lib-ambientcg-terms = Nemokama API ir atsisiuntimai; padėka laukiama, bet neprivaloma.

licence-lib-smithsonian-what = modeliai
licence-lib-smithsonian-licence = daugiausia CC0; kai kurie „Taikomos naudojimo sąlygos“ (nekomercinė)
licence-lib-smithsonian-terms = API nenurodo, kurie: reikia patikrinti modelio puslapį. Varstotas pažymi šiuos modelius kaip „patikrinkite naudojimą“ ir įspėja, kai jie padedami ar eksportuojami.

licence-lib-nasa-what = modeliai
licence-lib-nasa-licence = autorių teisėmis neginama (JAV viešasis turtas)
licence-lib-nasa-terms = NASA ženklai ir logotipai yra saugomi; jokiu naudojimu negalima sugestuoti NASA pritarimo.

licence-lib-kenney-what = modelių rinkiniai
licence-lib-kenney-licence = CC0
licence-lib-kenney-terms = Skaitoma iš viešų kenney.nl išteklių puslapių; Kenney logotipas nėra CC0.

licence-lib-quaternius-what = modelių rinkiniai
licence-lib-quaternius-licence = CC0
licence-lib-quaternius-terms = Atsisiunčiate patys naršyklėje.

licence-lib-search-name = Paieška svetainėse (Printables, Thingiverse, Sketchfab, …)

licence-lib-search-what = modeliai
licence-lib-search-licence = kiekvienas modelis turi savo
licence-lib-search-terms = Varstotas tik atveria svetainės paiešką jūsų naršyklėje; atsisiunčiate patys ir licenciją nustatote importo kortelėje.

licence-lib-plugins-name = Papildiniai

licence-lib-plugins-what = modeliai, medžiagos
licence-lib-plugins-licence = pagal papildinį
licence-lib-plugins-terms = Šaltinio papildinys rodo savo sąlygas (eilutė @notice) virš savo išteklių; pvz., Sketchfab reikia jūsų paties paskyros rakto ir padėkos autoriui bei Sketchfab.

## ======== Naudotojo vadovas ========

help-topic-start = Darbo pradžia
help-topic-navigation = Naršymas rodinyje
help-topic-selecting = Žymėjimas
help-topic-drawing = Piešimas
help-topic-shapes = 3D formos
help-topic-modifying = Keitimas
help-topic-inference = Išvedimas ir prilipimas
help-topic-measurements = Matmenų laukas
help-topic-groups = Grupės ir komponentai
help-topic-library = Komponentų biblioteka
help-topic-materials = Medžiagos ir tekstūros
help-topic-organizing = Žymės, struktūra, savybės
help-topic-construction = Pagalbinės linijos, matmenys, pjūviai
help-topic-styles = Stiliai ir vaizdo kūrimas
help-topic-scenes = Scenos, lapai, brėžiniai
help-topic-files = Failai, importas ir eksportas
help-topic-scripts = Scenarijai ir papildiniai
help-topic-shortcuts = Klaviatūra ir pelė
help-topic-problems = Kai kas nors negerai
## start
help-start-1 = Varstotas yra 3D modeliavimo programa: braižote briaunas, uždaros briaunos sudaro paviršius, paviršiai stumiami ir traukiami į kietus kūnus. Viskas lipdosi tarpusavyje, kol to nesugrupuojate.
help-start-2 = Langas
help-start-3 = Meniu juosta
help-start-4 = Failas, Redaguoti, Rodinys, Kamera, Piešimas, Įrankiai, Papildiniai, Langas, Pagalba
help-start-5 = Įrankių juosta
help-start-6 = įrankiai (paaiškinimas nurodo kiekvieną ir jo klavišą), naršymas ir Prilipimas; rodiniai ir stiliai yra rodinio viršutiniame dešiniajame kampe
help-start-7 = Rodinys
help-start-8 = modelis; raudona, žalia ir mėlyna linijos yra ašys, žemė po jomis
help-start-9 = Šoninė juosta
help-start-10 = kortelės dešinėje: Savybės, Medžiagos, Komponentai, Žymės, Struktūra, Scenos, Šešėliai, Aplinka, Švelninti briaunas, Patarimai (Langas → Šoninė juosta įjungia ir išjungia korteles)
help-start-11 = Būsenos juosta
help-start-12 = ko tikisi aktyvus įrankis, pranešimai ir klaidos, Žurnalo mygtukas, Matmenų laukas
help-start-13 = Pirmasis modelis
help-start-14 = 1. Paspauskite `R` (Stačiakampis), spustelėkite kampą ant žemės, spustelėkite priešingą kampą — arba įveskite `1000,600` ir Enter.
help-start-15 = 2. Paspauskite `P` (Stumti/Traukti), spustelėkite stačiakampį, patraukite aukštyn ir spustelėkite — arba įveskite `750` ir Enter.
help-start-16 = 3. Paspauskite `Space` (Žymėti), tris kartus spustelėkite dėžę, kad pažymėtumėte viską, paspauskite `G`, kad sukurtumėte grupę.
help-start-17 = 4. Paspauskite `B` (Dažų kibiras), Medžiagų kortelėje pasirinkite medžiagą, spustelėkite paviršius.
help-start-18 = 5. `Shift+Z` sutalpina modelį rodinyje; Failas → Išsaugoti įrašo `.vrst` failą.
help-start-19 = Pagalba dirbant
help-start-20 = Patarimų kortelė paaiškina aktyvų įrankį; F1 virš modelio atveria šį vadovą ties juo. Baigti modeliai apžiūrai (apstatytas namas, medžiagų galerija) yra aplanke { DATA_DIR() }/examples: Failas → Atidaryti.
## navigation
help-navigation-1 = Pelė ir jutiklinis kilimėlis
help-navigation-2 = Vidurinio mygtuko vilkimas
help-navigation-3 = sukti aplink tašką po žymekliu
help-navigation-4 = Shift + vidurinio mygtuko vilkimas
help-navigation-5 = slinkti
help-navigation-6 = Ratukas
help-navigation-7 = artinti link žymeklio
help-navigation-8 = Jutiklinis kilimėlis: braukimas dviem pirštais
help-navigation-9 = sukti; su Shift — slinkti
help-navigation-10 = Jutiklinis kilimėlis: suspaudimas arba %CMD% + braukimas
help-navigation-11 = artinti
help-navigation-12 = Tai veikia kiekviename įrankyje, todėl niekada nereikia nutraukti piešimo operacijos, kad apsidairytumėte.
help-navigation-13 = Langas → Nustatymai → Naršymas jas keičia: sukimo ir artinimo greitį, apverstą ratuką, artinimą link rodinio vidurio vietoje žymeklio, vidurinį mygtuką slinkimui (Shift sukimui), du pirštus slinkimui ar artinimui.
help-navigation-14 = Rodiniai
help-navigation-15 = Kamera → Standartiniai rodiniai (ir rodinio išplečiamasis meniu kampe): Izometrinis, Viršus, Apačia, Priekis, Galas, Kairė, Dešinė. Kamera → Lygiagreti projekcija / Perspektyva. `Shift+Z` — Priartinti iki ribų: visas modelis rodinyje.
help-navigation-16 = Išsaugoti rodiniai yra scenos — žr. Scenos, lapai, brėžiniai.
help-navigation-17 = Paties rodinio valdikliai
help-navigation-18 = Trys piktogramų išplečiamieji meniu rodinio viršutiniame dešiniajame kampe valdo viską, kaip jis rodo modelį (užveskite ant vieno, kad pamatytumėte, kas nustatyta):
help-navigation-19 = Rodinys
help-navigation-20 = standartiniai rodiniai, Perspektyva / Lygiagreti projekcija, Matymo laukas, Priartinti iki ribų, Rodiniai 1 / 2 / 4
help-navigation-21 = Stilius
help-navigation-22 = Šešėliuotas … Realistinis; Rentgeno, Briaunos, Kontūrai
help-navigation-23 = Rodyti (akis)
help-navigation-24 = ašys, žemė, dangus (išjungta = paprastas šviesus fonas), šio rodinio tinklelis; pagalbinės linijos, šešėliai, pjūvio plokštumos / pjūviai / užpildas modelio (visi rodiniai)
help-navigation-25 = Jungikliai palieka meniu atvertą; spustelėjimas už jo jį uždaro.
help-navigation-26 = Keli rodiniai
help-navigation-27 = Rodinys → Rodiniai (arba rodinio išplečiamasis meniu) rodo 1, 2 (greta) arba 4 (2 × 2) modelio vaizdus. Kiekvienas turi savo kamerą, stilių ir rodinio jungiklius; nauji pradeda nuo viršaus, priekio ir dešinės, lygiagrečia projekcija. Spustelėjimas rodinyje padaro jį aktyviu (mėlynas rėmelis): meniu, įrankių juostos ir įrankiai veikia jame, o pats spustelėjimas jame jau veikia.
help-navigation-28 = Šablonai (Rodinys → Rodiniai arba Rodinių šablonai rodinio išplečiamajame meniu) iš karto nustato įprastus išdėstymus, pvz., Priekis + Izometrinis arba Priekis, Kairė, Viršus + Izometrinis: plokšti rodiniai lygiagrečia projekcija, izometrinis perspektyva.
## selecting
help-selecting-1 = Redaguoti → Pažymėti viską (`%CMDK%A`), Nuimti žymėjimą (`%CMDK%T`). Delete arba Backspace ištrina pažymėtą; Redaguoti → Paslėpti paslepia, Rodyti viską grąžina viską.
help-selecting-2 = Spustelėkite modelį dešiniuoju pelės mygtuku, kad atvertumėte kontekstinį meniu: Ištrinti, Paslėpti, Sukurti grupę, Sukurti komponentą…, Išskaidyti, Susikertantys paviršiai, Apversti / Orientuoti paviršius, Apversti išilgai, Tekstūra → Padėtis, Išklostyti…, taip pat pjūvio plokštumų ir ašių komandos.
help-selecting-3 = Struktūros kortelė rodo grupes ir komponentus kaip medį: spustelėjimas ten pažymi, dvigubas spustelėjimas priartina prie objekto.
## drawing
help-drawing-1 = Piešimo įrankiai veikia paviršiuje po žymekliu arba ant žemės / aktyvios darbo plokštumos. Uždari briaunų kontūrai tampa paviršiais; linija per paviršių jį padalija.
help-drawing-2 = Kol įrankis veikia, bet kada galite įvesti reikšmę (nereikia spustelėti Matmenų lauko) ir paspausti Enter — žr. Matmenų laukas.
## shapes
help-shapes-1 = Braižyti → Formos sukuria visą kietą kūną viena operacija. Rezultatas yra sugeneruotas komponentas: pažymėkite jį ir keiskite jo dydžius bei segmentus Savybėse; Išskaidyti paverčia jį paprasta geometrija.
## modifying
help-modifying-1 = Perkelti, Sukti ir Keisti mastelį veikia pažymėtą dalyką arba tai, ką spustelėjate, kai nieko nepažymėta. Perkeliant viršūnę, briauną ar paviršių, aplink esanti geometrija ištempiama.
help-modifying-2 = Kopijavimas
help-modifying-3 = Perkelti / Sukti su %COPY% sukuria kopiją
help-modifying-4 = Masyvai
help-modifying-5 = po kopijos įveskite `5x` (penkios kopijos tuo pačiu žingsniu) arba `/5` (penkios kopijos tarp)
help-modifying-6 = Redaguoti meniu ir kontekstinis meniu
help-modifying-7 = Susikertantys paviršiai
help-modifying-8 = su modeliu, pažymėtu dalyku ar kontekstu: briaunos, kur paviršiai kertasi
help-modifying-9 = Pjauti
help-modifying-10 = plokštuma perpjauna pažymėtą dalyką ar kontekstą; su %COPY% (Skaidyti) kiekvienas perpjautas kūnas tampa dviem uždarom grupėm
help-modifying-11 = Apversti / Orientuoti paviršius
help-modifying-12 = pasukti priekinę pusę (baltą) į išorę
help-modifying-13 = Apversti išilgai
help-modifying-14 = atspindėti pažymėtą dalyką išilgai raudonos, žalios ar mėlynos krypties
help-modifying-15 = Švelninti briaunas kortelė
help-modifying-16 = švelninti / glotninti briaunas pagal kampą
help-modifying-17 = Langas → Išklostyti
help-modifying-18 = išklostyti paviršius ar grupes į iškirpimo šabloną (SVG / PDF / DXF)
## inference
help-inference-1 = Piešiant žymeklis prilimpa prie modelio taškų ir krypčių; ženklas ir paaiškinimas parodo, ką rado.
help-inference-2 = Galinis taškas, Vidurio taškas
help-inference-3 = žalias / žydras taškas
help-inference-4 = Ant briaunos, Ant paviršiaus
help-inference-5 = raudonas kvadratas / mėlynas rombas
help-inference-6 = Centras, Sankirta
help-inference-7 = apskritimų ir lankų, briaunų
help-inference-8 = Ant raudonos / žalios / mėlynos ašies
help-inference-9 = ašies spalvos linija
help-inference-10 = Lygiagrečiai / Statmenai
help-inference-11 = purpurinė linija išilgai briaunos
help-inference-12 = Nuo taško
help-inference-13 = punktyrinė linija nuo taško išilgai ašies
help-inference-14 = Užrakinimas
help-inference-15 = Laikykite `Shift`, kad užrakintumėte rastą kryptį; rodyklių klavišai užrakina ašį: → raudona, ← žalia, ↑ mėlyna, ↓ lygiagrečiai / statmenai paskutinei briaunai.
help-inference-16 = Prilipimo meniu
help-inference-17 = Magneto mygtukas (ir Prilipimo meniu) įjungia ar išjungia kiekvieną išvedimą; `Shift+S` išjungia ir įjungia prilipimą visiškai. Žingsniai apvalina laisvus perkėlimus (10 mm), kampus (15°) ir mastelio koeficientus — prilipimas prie geometrijos ir įvestos reikšmės turi pirmenybę.
help-inference-18 = Rodinys → Tinklelis rodo tinklelį su savo prilipimu; žingsnis yra Langas → Nustatymai → Piešimas, o kai aktyvus Žymėti, įvedus ilgį jis nustatomas.
help-inference-19 = Darbo plokštumos
help-inference-20 = Pagalbinės plokštumos įrankis (Įrankių meniu) ir Langas → Darbo plokštumos sukuria plokštumą piešti ore: taškai tuščioje erdvėje nusileidžia ant jos, ašys ir tinklelis seka paskui ją.
## measurements
help-measurements-1 = Matmenų laukas būsenos juostos dešinėje rodo dabartinį ilgį, kampą ar dydį. Tiesiog renkite, kol įrankis veikia, ir paspauskite Enter; Backspace taiso, Esc išvalo.
help-measurements-2 = 1200, 1.2m, 120cm, 4', 1'6"
help-measurements-3 = ilgiai: modelio vienetas arba jūsų nurodytas vienetas
help-measurements-4 = 1000,600
help-measurements-5 = dvi reikšmės: stačiakampio kraštinės, dėžės pagrindas
help-measurements-6 = 45
help-measurements-7 = kampas (Sukti, Transporteris, lankai)
help-measurements-8 = 24s
help-measurements-9 = apskritimo, lanko ar formos segmentai; daugiakampio kraštinės
help-measurements-10 = 5x, /5
help-measurements-11 = kopijos eilėje po Perkelti / Sukti kopijos
help-measurements-12 = 2, 0.5
help-measurements-13 = mastelio koeficientas
help-measurements-14 = Vėl įvedus iškart po operacijos, ta operacija pakeičiama (kitas ilgis, daugiau kopijų).
help-measurements-15 = Modelio vienetas imamas iš šablono (Failas → Naujas iš šablono); kaip dešimtainis skyriklis tinka ir `,`, ir `.`.
## groups
help-groups-1 = Geometrija lipdosi: besiliečiančios briaunos ir paviršiai susijungia ir vienas kitą dalija. Kad dalis liktų atskira, paverskite ją grupe arba komponentu.
help-groups-2 = Sukurti grupę (G)
help-groups-3 = grupė: vienas objektas; kopijos nepriklausomos
help-groups-4 = Sukurti komponentą…
help-groups-5 = komponentas: kopijos dalijasi vienu apibrėžimu — redaguojant vieną, redaguojamos visos
help-groups-6 = Dukart spustelėti
help-groups-7 = įeiti į grupę / komponentą redaguoti (likusi dalis pritemsta); Esc arba Redaguoti → Uždaryti grupę išeina
help-groups-8 = Išskaidyti
help-groups-9 = atgal į laisvą geometriją
help-groups-10 = Padaryti unikalų (dešinysis spustelėjimas, Savybės)
help-groups-11 = komponento kopija gauna savo apibrėžimą („Dėžė 2“): jos redagavimas nebekeičia kitų kopijų; sugeneruotas gauna savo generatoriaus kopiją („Lentyna 2“ skiltyje Modelyje)
help-groups-12 = Vėl padėti iš bibliotekos
help-groups-13 = modelyje esanti kopija naudojama pakartotinai, kol nepakeista; ją suredagavus, bibliotekos originalas ateina kaip naujas apibrėžimas („Kėdė 2“)
help-groups-14 = Komponentai su parametrais
help-groups-15 = Redaguoti → Padaryti parametrinį suteikia komponentui dydžius X / Y / Z (tempti ar keisti mastelį) ir kopijų eilę Savybėse.
help-groups-16 = Langas → Komponento parametrai suteikia bet kuriam komponentui parametrus: reikšmes ir formules, kurios sukioja, stumdo, rodo ar dažo jo dalis, pasirinktinį scenarijų; kiekviena kopija turi savo reikšmes (slankikliai Savybėse).
help-groups-17 = Generatoriai yra komponentai, sukurti scenarijaus su tipizuotais parametrais (Langas → Generatorių redaktorius). Braižyti → Formos kūnai taip pat yra generatoriai.
help-groups-18 = Komponento scenarijus Scenarijų žinyne
help-groups-19 = Generatoriai Scenarijų žinyne
## library
help-library-1 = Komponentų ir Medžiagų kortelės atrodo vienodai: viršutinė eilutė (Ieškoti, Atnaujinti, + pridėti, debesies mygtukas internetinei bibliotekai, … bibliotekos aplankui, ką rodyti ir plytelių dydis: Mažos / Didelės / Sąrašas), tada skiltys Modelyje (atvertas dokumentas), Mėgstami, Neseniai, Mano biblioteka su aplankais ir Pavyzdžiai. Skiltis sutraukiama rodykle; aplanko pavadinimas atveria jį Finder.
help-library-2 = Dešinysis spustelėjimas ant plytelės → Pridėti prie mėgstamų
help-library-3 = ☆ plytelė taip pat rodoma viršuje skiltyje Mėgstami (Pašalinti iš mėgstamų ją išima); Neseniai išvardija, ką neseniai padėjote ar pasirinkote dažyti
help-library-4 = Dešinysis spustelėjimas ant aplanko pavadinimo
help-library-5 = Naujas aplankas…; poaplankiui taip pat Pervadinti aplanką… ir Ištrinti aplanką (tik kai tuščias)
help-library-6 = Dešinysis spustelėjimas ant plytelės → Perkelti į aplanką
help-library-7 = perkelti failą į kitą bibliotekos aplanką (mėgstami seka paskui)
help-library-8 = Vilkti plytelę į rodinį
help-library-9 = komponentas padedamas ten, kur paleidžiate; medžiaga nudažo paviršių po žymekliu
help-library-10 = Komponentai rodo jūsų biblioteką (`{ DATA_DIR() }/components` arba aplanką, kurį pasirenkate su …), generatorius, pridėtus Pavyzdžius ir atverto dokumento komponentus.
help-library-11 = Spustelėti plytelę
help-library-12 = padėti ją: kopija seka žymeklį, spustelėkite, kad nuleistumėte
help-library-13 = + → Išsaugoti pažymėtą bibliotekoje…
help-library-14 = įrašyti pažymėtą kaip naują `.vrst` failą
help-library-15 = Dešinysis spustelėjimas ant plytelės
help-library-16 = Padėti, Nustatyti peržiūrą iš rodinio, Nustatyti peržiūros vaizdą…, Pašalinti peržiūros vaizdą, Pervadinti…, Rodyti Finder, Ištrinti…; dokumento komponentui taip pat Pažymėti egzempliorius, Išsaugoti bibliotekoje, Parametrai…, Ištrinti iš dokumento…
help-library-17 = Paveikslėliai
help-library-18 = Plytelė rodo failo paveikslėlį, jei jis yra, kitaip — sugeneruotą miniatiūrą. Nustatyti peržiūrą iš rodinio paima rodinį tokį, koks jis dabar — pirmiausia sukadruokite komponentą. Iš bibliotekos padėtas komponentas pasiima failo paveikslėlį su savimi; Failas → Peržiūros vaizdas nustato atverto dokumento paveikslėlį.
help-library-19 = Internetinė 3D biblioteka
help-library-20 = Debesies mygtukas Komponentų kortelėje (arba Langas → Internetinė 3D biblioteka) atsisiunčia modelius: Poly Haven, Smithsonian, NASA, Kenney rinkinius, pagal URL arba iš papildinių šaltinių. Atsisiųstas modelis tampa bibliotekos komponentu su savo padėka (Savybės → Šaltinis, Failas → Eksportuoti → Priskyrimai). Filtrai pagal licenciją, formatą ir dydį; mėgstami ir atsisiuntimų istorija.
help-library-21 = Kiekvienas šaltinis rodo savo sąlygas virš modelių (papildinio eilutė @notice). Modelis, kurio licencija riboja naudojimą (nekomercinė, be išvestinių darbų, dalytis tomis pačiomis sąlygomis, paties tinklalapio licencija, nežinoma), gauna įspėjimą jį padedant ir eksportuojant dokumentą. Pagalba → Licencijos… išvardija Varstotas licenciją, atvirojo kodo komponentus, failų formatus ir internetines bibliotekas su jų sąlygomis.
## materials
help-materials-1 = Medžiagų kortelė rodo modelio medžiagas (Modelyje), jūsų biblioteką (`{ DATA_DIR() }/materials` arba aplanką, kurį pasirenkate su …) ir Pavyzdžius; dešiniuoju mygtuku spustelėkite plytelę: Naudoti, Pervadinti…, Išsaugoti bibliotekoje, Rodyti Finder, Eksportuoti…, Ištrinti…. Pasirinkus bibliotekos medžiagą, tik parengiamas Dažų kibiras: medžiaga prisijungia prie modelio (Modelyje) nudažius pirmąjį paviršių. Ištrinti ant plytelės Modelyje pašalina medžiagą iš modelio, jos paviršiai grįžta į numatytąją (Redaguoti → Atšaukti ją grąžina). Medžiagą redaguokite ten: spalva, nepermatomumas, tekstūros vaizdas ir jo dydis, fizinės savybės Realistiniam stiliui (metalas, šiurkštumas, stiklas, spinduliavimas, žemėlapiai, danga, blizgesys).
help-materials-2 = Išsaugoti bibliotekoje / Eksportuoti… / + → Importuoti…
help-materials-3 = dalytis medžiaga tarp dokumentų (`.vrstmat`)
help-materials-4 = Debesies mygtukas (Medžiagų kortelė)
help-materials-5 = atsisiųsti PBR skenavimus iš Poly Haven ir ambientCG arba iš papildinių šaltinių
help-materials-6 = Šešėliuoklė
help-materials-7 = medžiaga gali turėti savo WGSL šešėliuoklę su parametrais (šešėliuoklių redaktorius turi savo žinyną)
help-materials-8 = Paviršiai turi priekinę (baltą) ir galinę (pilkai mėlyną) puses; grupės dažymas nudažo jos viduje esančius paviršius, neturinčius savo medžiagos.
## organizing
help-organizing-1 = Savybės
help-organizing-2 = pažymėto dalyko savybės: plotas, ilgis, kietųjų kūnų tūris, žymė, medžiaga, pavadinimas, komponentų ir generatorių parametrai
help-organizing-3 = Žymės
help-organizing-4 = žymių (sluoksnių) matomumas, spalva ir pavadinimas; ištrynus žymę, jos turinys perkeliamas į Be žymės
help-organizing-5 = Struktūra
help-organizing-6 = grupių ir komponentų medis: Priartinti prie, Redaguoti, Paslėpti, Išskaidyti, Iškelti iš grupės, Ištrinti; vilkite eilutes, kad perkeltumėte objektus tarp grupių
help-organizing-7 = Paslėpti / Rodyti viską
help-organizing-8 = Langas → Dokumento informacija
help-organizing-9 = dokumento statistika (paviršiai, trikampiai, komponentai, medžiagos, tekstūros …) ir Pašalinti nenaudojamus
## construction
help-construction-1 = Pagalbinės linijos žymimos, perkeliamos ir trinamos kaip geometrija; Redaguoti → Ištrinti pagalbines linijas pašalina jas visas, Rodinys → Pagalbinės linijos paslepia. Įrankiai → 3D tekstas… sukuria tekstą kaip geometriją.
help-construction-2 = Rodinys → Pjūvio plokštumos / Pjūviai / Pjūvio užpildas valdo, kaip rodomi pjūviai; aktyvus pjūvis išsaugomas scenose.
## styles
help-styles-1 = Rodinys meniu (ir rodinio išplečiamasis meniu Stilius)
help-styles-2 = Šešėliuotas, Šešėliuotas su tekstūromis, Vienspalvis
help-styles-3 = paviršiai medžiagų spalvomis, su tekstūromis, viena spalva
help-styles-4 = Paslėptos linijos, Karkasas
help-styles-5 = tik briaunos (paslėptos pašalintos arba ne)
help-styles-6 = Realistinis
help-styles-7 = fizinės medžiagos, apšvietimas iš aplinkos, saulė, efektai — Aplinkos kortelė nustato dangų, HDRI, ekspoziciją ir efektus
help-styles-8 = Rentgeno, Briaunos, Kontūrai
help-styles-9 = matyti pro paviršius; briaunos ir storos kontūro linijos įjungiamos ir išjungiamos
help-styles-10 = Šešėliai
help-styles-11 = saulės šešėliai; laikas ir data yra Šešėlių kortelėje
help-styles-12 = Vaizdo kūrimas
help-styles-13 = Langas → Vaizdo kūrimas sukuria rodinio vaizdą spindulių sekimu GPU: stiklas, atspindžiai, šviesos, švelnūs šešėliai; kokybės šablonai, kadro dydis, lauko gylis, triukšmo šalinimas (kai įdiegtas Intel OIDN), spalvų korekcija; PNG / EXR arba visos scenos iš karto.
help-styles-14 = Šviesos (taškinė, prožektoriaus, srities) padedamos iš Vaizdo kūrimo lango; jos pažymimos spustelėjus jų ženklą, perkeliamos ir sukamos kaip kiti objektai.
help-styles-15 = Failas → Eksportuoti → 2D grafika (PNG) išsaugo rodinį toks, koks jis yra; Failas → Eksportuoti → Animacija… sukuria animuotą GIF ar PNG kadrus iš scenų arba apsisukimo aplink.
## scenes
help-scenes-1 = Scenų kortelė
help-scenes-2 = išsaugoti rodiniai su savo stiliumi, matomomis žymėmis ir pjūviu; skirtukai virš rodinio juos perjungia
help-scenes-3 = Rodinys → Animacija
help-scenes-4 = Groti scenas; PgDn / PgUp — kita / ankstesnė scena
help-scenes-5 = Langas → Lapai
help-scenes-6 = maketavimo puslapiai: scenų rodiniai mastelio 1:N, tekstai, pagrindinis užrašas, standartiniai rodiniai su matmenimis, dalių lentelė; kelių puslapių PDF
help-scenes-7 = Failas → Eksportuoti → 2D brėžinys
help-scenes-8 = dabartinis rodinys kaip vektorinis SVG / PDF su kontūrais, paslėptomis linijomis, pjūviais ir matmenimis
help-scenes-9 = Failas → Eksportuoti → Ataskaita
help-scenes-10 = dalys su pjovimo dydžiais, plotai pagal medžiagą (CSV)
## files
help-files-1 = Dokumentai yra `.vrst` failai (tekstūros ir paveikslėliai išsaugomi viduje). Failas → Naujas iš šablono pasirenka vienetus; Langas → Nustatymai nustato numatytąjį šabloną ir automatinio išsaugojimo intervalą.
help-files-2 = Langas → Nustatymai turi skiltis kairėje ir paiešką: Bendra (automatinis išsaugojimas, Failas → Naujas šablonas, kiek atšaukimo žingsnių laikoma), Sąsaja (šviesi, tamsi ar sistemos tema, meniu ir skydelių mastelis, įrankių juostos piktogramų dydis), Piešimas (tinklelis, prilipimas ir žingsniai — tie patys jungikliai kaip Prilipimo meniu), Bibliotekos (komponentų ir medžiagų aplankai, Pavyzdžiai, plytelių dydis), Atsisiuntimai (stebimas aplankas), Naršymas (pelė ir jutiklinis kilimėlis), Spartieji klavišai (spustelėkite sparčiąjį klavišą ir paspauskite naujus klavišus; Backspace jį pašalina, ↺ grąžina numatytąjį), Vaizdo kūrimas (naujų dokumentų vaizdo kūrimo nustatymai; kur yra Open Image Denoise biblioteka) ir Rodinys (kaip atrodo naujas dokumentas: stilius, briaunos, ašys, žemė, dangus, tinklelis, šešėliai, projekcija, matymo laukas; Naudoti dabartinį rodinį / Taikyti dabartiniam rodiniui). Pakeitimai taikomi iš karto; Atkurti grąžina skiltį į numatytąsias reikšmes.
help-files-3 = Automatinis išsaugojimas įrašo neišsaugotus pakeitimus į `{ DATA_DIR() }/autosave`; po strigties Varstotas pasiūlo juos atkurti.
help-files-4 = Importas
help-files-5 = OBJ, STL, DXF, glTF / GLB, 3MF, COLLADA (DAE), FBX, PLY, taškai iš CSV ir formatai, pridėti papildinių; failus taip pat galite numesti ant lango
help-files-6 = Eksportas
help-files-7 = OBJ, STL, DXF, glTF (GLB), 3MF, COLLADA (DAE), PLY, paviršiai kaip JSON, PNG, SVG / PDF brėžiniai, lapai, išklotinės, animacija, ataskaita, priskyrimai
help-files-8 = Redaguoti → Iškirpti / Kopijuoti / Įklijuoti / Įklijuoti toje pačioje vietoje
help-files-9 = Įklijuoti toje pačioje vietoje išlaiko nukopijuotą padėtį
help-files-10 = Failas → Peržiūros vaizdas
help-files-11 = paveikslėlis, kurį biblioteka rodo šiam failui
## scripts
help-scripts-1 = Langas → Scenarijų konsolė vykdo Rhai scenarijus modelyje: kurti, keisti, žymėti, matuoti. Failas → Paleisti scenarijų… paleidžia `.rhai` (arba `.py`) failą. Vienas paleidimas yra vienas atšaukimo žingsnis.
help-scripts-2 = Papildiniai yra scenarijai aplanke `{ DATA_DIR() }/plugins`: jie tampa Papildiniai meniu elementais su parametrų ir taškų parinkimo dialogais; jie gali pridėti importuotuvus, eksportuotuvus ir medžiagų / modelių šaltinius.
help-scripts-3 = Python ir DI agentai gali valdyti Varstotas per vietinį tiltą (Python klientas, MCP serveris).
help-scripts-4 = Atverti Scenarijų žinyną
help-scripts-5 = Konsolė ir papildiniai
help-scripts-6 = Visos funkcijos
## shortcuts
help-shortcuts-1 = Įrankių klavišai veikia, kai joks teksto laukas neturi fokuso. Operacijos metu surinktos raidės patenka į Matmenų lauką.
## problems
help-problems-1 = Langas → Žurnalas
help-problems-2 = kiekviena seanso klaida ir įspėjimas — scenarijai, importas / eksportas, bibliotekos, atsisiuntimai; raudonas skaitiklis būsenos juostoje reiškia naujas klaidas
help-problems-3 = Redaguoti → Atšaukti
help-problems-4 = kiekvieną operaciją galima atšaukti; nepavykusi operacija nieko nekeičia
help-problems-5 = Langas → Dokumento informacija → Pašalinti nenaudojamus
help-problems-6 = pašalinti nenaudojamus komponentus, medžiagas ir žymes, kad failas būtų mažesnis
help-problems-7 = Paviršiai atrodo pilkai mėlyni
help-problems-8 = matote jų galinę pusę: pažymėkite juos, dešinysis spustelėjimas → Apversti paviršius arba Orientuoti paviršius
help-problems-9 = Negalima piešti ten, kur tikėtasi
help-problems-10 = patikrinkite Shift / rodyklių užraktus, aktyvią darbo plokštumą (Langas → Darbo plokštumos) ir ar nesate grupės viduje (Esc iš jos išeina)
help-problems-11 = Kažko trūksta
help-problems-12 = Redaguoti → Rodyti viską, Žymių kortelė (paslėptos žymės), Rodinys → Pjūvio plokštumos

## Langas, paieška, spartieji klavišai
help-window-title = Naudotojo vadovas
help-search = Ieškoti
help-search-hint = įrankis, komanda ar žodis
help-nothing-found = Nieko nerasta
help-in-the-guide = Vadove
help-tools = Įrankiai
help-commands = Komandos
help-use = Naudoti
help-use-hint = Pasirinkti šį įrankį
help-shortcuts-note = Keiskite juos Langas → Nustatymai → Spartieji klavišai. Pelė ir jutiklinis kilimėlis: žr. Naršymas rodinyje. Žymėjimo modifikatoriai: žr. Žymėjimas.
help-key-cut-copy-paste-keys = %CMDK%X / %CMDK%C / %CMDK%V
help-key-cut-copy-paste = Iškirpti / Kopijuoti / Įklijuoti
help-key-delete = Delete, Backspace
help-key-delete-what = ištrinti pažymėtą
help-key-esc = Esc
help-key-esc-what = atšaukti operaciją; išeiti iš redaguojamos grupės
help-key-shift = Shift (laikyti)
help-key-shift-what = užrakinti išvedimo kryptį
help-key-arrows = → ← ↑ ↓
help-key-arrows-what = užrakinti raudoną / žalią / mėlyną ašį, lygiagrečiai paskutinei briaunai
help-key-enter = Enter
help-key-enter-what = taikyti Matmenų lauke įvestą reikšmę
help-key-f1 = F1
help-key-f1-what = šis vadovas apie aktyvų įrankį; scenarijų redaktoriuje — Scenarijų žinynas

## ======== Scenarijai ========

## Funkcijos
script-fn-vec3 = Taškas / vektorius (tinka ir masyvas [x, y, z]).
script-fn-line = Briaunos nuo a iki b.
script-fn-polyline = Briaunos per taškus; uždara ir plokščia → paviršius.
script-fn-face = Paviršius per taškus.
script-fn-rect = Stačiakampis paviršius XY plokštumoje.
script-fn-circle = Apskritimo paviršius (arba briaunos).
script-fn-polygon = Taisyklingojo daugiakampio paviršius.
script-fn-arc = Lanko briaunos aplink centrą.
script-fn-box = Nauja gretasienio grupė (pradžia = mažiausias kampas).
script-fn-cylinder = Nauja cilindro grupė (apačia ties centru).
script-fn-cone = Nauja kūgio grupė (pagrindas ties centru, viršūnė h aukščiau).
script-fn-pyramid = Nauja piramidės grupė: w × d pagrindas nuo pradžios taško, viršūnė h virš jo vidurio.
script-fn-sphere = Nauja sferos grupė aplink centrą.
script-fn-torus = Nauja toro grupė aplink centrą: r iki vamzdžio vidurio, tube = jo spindulys.
script-fn-pushpull = Stumti / traukti paviršių; grąžina viršutinį paviršių.
script-fn-offset = Paviršiaus kontūro nuokrypis (d < 0 į vidų); grąžina naują paviršių.
script-fn-follow-me = Profilio paviršiaus ištempimas išilgai trajektorijos.
script-fn-erase = Ištrinti objektą arba jų masyvą.
script-fn-move = Perkelti objektus vektoriumi.
script-fn-copy = Kopijuoti objektus vektoriumi; grąžina kopijas.
script-fn-rotate = Pasukti objektus apie ašį.
script-fn-scale = Keisti objektų mastelį apie tašką.
script-fn-group = Sugrupuoti objektus; grąžina grupę.
script-fn-component = Sukurti komponentą; grąžina jo egzempliorių.
script-fn-explode = Išskaidyti grupę / komponentą.
script-fn-make-unique = Suteikti komponento egzemplioriui atskirą apibrėžtį.
script-fn-union = Dviejų kietųjų grupių kietųjų kūnų sąjunga.
script-fn-subtract = Kietasis kūnas a minus b.
script-fn-intersect = Kietųjų kūnų sankirta.
script-fn-trim = a minus b, b lieka.
script-fn-split = Padalyti du kietuosius kūnus į tris.
script-fn-outer-shell = Kietųjų kūnų išorinis apvalkalas.
script-fn-reverse = Apversti paviršius.
script-fn-soften = Sušvelninti briaunas iki lūžio kampo.
script-fn-material = Sukurti / perspalvinti medžiagą; grąžina jos pavadinimą.
script-fn-paint = Dažyti paviršius / grupes (() pašalina).
script-fn-tag = Priskirti žymę (sukuriama, jei jos nėra).
script-fn-make-parametric = Padaryti grupę parametrinę.
script-fn-set-params = Nustatyti sugeneruoto komponento reikšmes arba parametrinės grupės dydį / kopijas.
script-fn-params = Sugeneruoto komponento reikšmės / parametrinės grupės parametrai arba ().
script-fn-reset-params = Sugeneruotas komponentas grįžta prie generatoriaus numatytųjų reikšmių.
script-fn-generators = Dokumento generatorių pavadinimai.
script-fn-add-generator = Pridėti / atnaujinti generatorių iš jo šaltinio; grąžina jo pavadinimą.
script-fn-generate = Įdėti sugeneruotą komponentą taške.
script-fn-dimension = Tiesinis matmuo tarp taškų.
script-fn-note = Teksto pastaba su išnaša.
script-fn-enter = Nuo šiol kurti grupės viduje.
script-fn-exit = Išeiti iš paskutinės įeitos grupės.
script-fn-faces = Dabartinio konteksto paviršiai.
script-fn-edges = Dabartinio konteksto briaunos.
script-fn-groups = Dabartinio konteksto grupės / komponentai.
script-fn-all = Viskas dabartiniame kontekste.
script-fn-entities = Viskas grupės viduje.
script-fn-child = Grupė / komponentas pavadinimu name dabartiniame kontekste (arba g viduje); deep = ieškoti ir įdėtosiose grupėse. () jei nerasta.
script-fn-children = Dabartinio konteksto (arba g viduje) grupės / komponentai; deep = ir įdėtieji.
script-fn-show = Rodyti objektus (on = false: slėpti).
script-fn-hide = Slėpti objektus.
script-fn-selection = Pasirinkimas.
script-fn-select = Nustatyti pasirinkimą (() išvalo).
script-fn-bounds = #{ "{" } min, max, size, center { "}" } pasaulio koordinatėmis.
script-fn-model-bounds = Viso modelio ribos.
script-fn-length = Perskaityti ilgį su vienetais ("1.2m") → mm.
script-fn-units = Modelio ilgio vienetas.
script-fn-materials = Medžiagų pavadinimai.
script-fn-all-faces = Visi paviršiai, įskaitant įdėtąsias grupes.
script-fn-read-text = Perskaityti tekstinį failą.
script-fn-lang = Sąsajos kalbos kodas ("en", "ru", …): pasirinkti tekstus, kuriuos įskiepis spausdina ar rodo.
script-fn-platform = "macos", "windows" arba "linux".
script-fn-write-text = Įrašyti tekstinį failą.
script-fn-file-exists = Failas ar aplankas egzistuoja.
script-fn-list-files = Failai aplanke dir (ir poaplankiuose) su plėtiniu ("" = bet kuriuo), surikiuoti.
script-fn-list-dirs = Aplanko dir aplankai, surikiuoti.
script-fn-parse-json = JSON tekstas → žemėlapiai, masyvai, skaičiai, eilutės.
script-fn-fetch-text = Atsisiųsti tinklalapį / API atsakymą kaip tekstą (HTTP(S)); headers: #{ "{" } "X-Auth-Token": key { "}" }.
script-fn-fetch-file = Atsisiųsti url į failą path.
script-fn-url-encode = Užkoduoti tekstą procentais URL adresui (paieškos užklausai).
script-fn-print = Rašyti į konsolės žurnalą.
script-fn-len = Masyvo / eilutės ilgis.
script-fn-range = Sveikieji skaičiai nuo from iki to (to neįskaitant), ciklams.
script-fn-min = Mažesnis skaičius.
script-fn-max = Didesnis skaičius.
script-fn-clamp = x, laikomas ribose lo..hi.
script-fn-ceil = Apvalinti aukštyn iki sveikojo skaičiaus (kaip float).
script-fn-noise = Glotnus triukšmas 0..1 taške x, y (viena ląstelė = 1). Kai period > 0, kartojasi kas period ląstelių (vientisos plytelės); seed parenka raštą.
script-fn-fbm = Fraktalinis triukšmas 0..1: triukšmo oktavos, kiekviena dvigubai smulkesnė ir dvigubai silpnesnė; period ir seed – kaip noise.

## Skyriai
script-group-points-and-vectors = Taškai ir vektoriai
script-group-drawing = Braižymas
script-group-solids = Kietieji kūnai
script-group-editing = Redagavimas
script-group-groups-and-components = Grupės ir komponentai
script-group-finding-objects = Objektų paieška
script-group-visibility-materials-tags = Matomumas, medžiagos, žymės
script-group-solid-tools = Kietųjų kūnų įrankiai
script-group-parameters-and-generators = Parametrai ir generatoriai
script-group-dimensions-and-notes = Matmenys ir pastabos
script-group-files-and-network = Failai ir tinklas
script-group-output-and-numbers = Išvestis ir skaičiai

## Objektų, vektorių, masyvų, eilučių nariai
script-member-kind = "face", "edge", "group", "component" …
script-member-valid = objektas vis dar egzistuoja
script-member-area = paviršiaus plotas, mm²
script-member-normal = paviršiaus normalė (pasaulio)
script-member-edges = paviršiaus briaunos
script-member-faces = briaunos paviršiai / paviršiaus kaimynai
script-member-length = briaunos ilgis, mm
script-member-soft = briauna švelni
script-member-center = ribų centras (pasaulio)
script-member-vertices = paviršiaus / briaunos taškai (pasaulio)
script-member-name = grupės / komponento pavadinimas (priskiriamas)
script-member-hidden = objektas paslėptas (priskiriamas)
script-member-bounds = #{ "{" } min, max, size, center { "}" } (pasaulio)
script-member-size = ribų dydis pagal x, y, z
script-member-min = mažiausias ribų kampas (pasaulio)
script-member-max = didžiausias ribų kampas (pasaulio)
script-member-definition = komponento apibrėžties pavadinimas
script-member-material = medžiagos pavadinimas arba ()
script-member-tag = žymės pavadinimas arba ()
script-member-x = x koordinatė (priskiriama)
script-member-y = y koordinatė (priskiriama)
script-member-z = z koordinatė (priskiriama)
script-member-length-2 = vektoriaus ilgis
script-member-normalize = vienetinis vektorius
script-member-dot = skaliarinė sandauga
script-member-cross = vektorinė sandauga
script-member-distance = atstumas iki b
script-member-min-2 = mažiausias kampas
script-member-max-2 = didžiausias kampas
script-member-size-2 = dydis pagal x, y, z
script-member-center-2 = centras
script-member-len = elementų skaičius
script-member-is-empty = nėra elementų
script-member-push = pridėti elementą
script-member-pop = pašalinti ir grąžinti paskutinį elementą
script-member-insert = įterpti elementą
script-member-remove = pašalinti ir grąžinti elementą
script-member-contains = masyvas / eilutė turi reikšmę
script-member-index-of = reikšmės indeksas arba -1
script-member-filter = elementai, kuriems uždarinys teisingas
script-member-map = naujas uždarinio rezultatų masyvas
script-member-reduce = sutraukti elementus
script-member-len-2 = simbolių skaičius
script-member-to-upper = didžiosios raidės (vietoje)
script-member-to-lower = mažosios raidės (vietoje)
script-member-trim = pašalinti tarpus (vietoje)
script-member-split = padalyti į masyvą
script-member-starts-with = eilutė prasideda tekstu
script-member-to-string = kaip tekstas

## Įskiepių antraščių žymės
script-tag-name = pavadinimas meniu ir dialogo lango antraštėje
script-tag-menu = kelias skiltyje Papildiniai
script-tag-description = paaiškinimas ir dialogo lango tekstas
script-tag-param = reikšmė, klausiama dialogo lange prieš vykdymą
script-tag-pick = N taškų, spustelėtų prieš vykdymą → points
script-tag-select = objektai, pasirinkti prieš vykdymą → objects
script-tag-import = įrašas Failas → Importuoti; failas kintamajame path
script-tag-export = įrašas Failas → Eksportuoti; failas kintamajame path
script-tag-generator = generatorius: paverčia šį failą generatoriumi (žr. docs/GENERATORS.md)
script-tag-formula = generatorius: iš parametrų apskaičiuota reikšmė, kintamajame p
script-tag-check = generatorius: reikšmės, kurioms ji klaidinga, atmetamos

## Parametrų tipai (@param key:type)
script-param-type-length = ilgis su vienetais ("1.2m"), scenarijuje mm — numatytasis skaičiams
script-param-type-int = sveikasis skaičius
script-param-type-number = trupmeninis skaičius
script-param-type-text = tekstas — numatytasis reikšmei kabutėse
script-param-type-bool = žymimasis langelis — numatytasis true / false
script-param-type-choice = generatorius: viena iš choice(a|b|c)
script-param-type-angle = generatorius: laipsniai, slankiklis
script-param-type-percent = generatorius: 0–100, slankiklis
script-param-type-material = generatorius: dokumento medžiagos pavadinimas

script-type-face = paviršius
script-type-edge = briauna
script-type-group = grupė
script-type-vector = vektorius
script-type-num = skaičius
script-type-bool = bool
script-type-str = eilutė
script-type-bounds = ribos
script-type-faces = paviršių masyvas
script-type-edges = briaunų masyvas
script-type-groups = grupių masyvas
script-type-vecs = taškų masyvas
script-type-array = masyvas
script-type-any = objektas / vektorius

## Parametrų tipai be pradžios „generatorius: “ (Scenarijų žinynas, generatoriaus tema)
script-param-type-choice-plain = viena iš choice(a|b|c)
script-param-type-angle-plain = laipsniai, slankiklis
script-param-type-percent-plain = 0–100, slankiklis
script-param-type-material-plain = dokumento medžiagos pavadinimas

# Klaidos, kurias mato scenarijaus naudotojas (API, generatoriai, parametrų konstrukcijos)

## api.rs
script-err-k-expected-a-number-bool-or = { $k }: tikėtasi skaičiaus, bool arba eilutės, gauta { $type_name }
script-err-set-params-unknown-key-k-x = set_params: nežinomas raktas { $k } (x, y, z, count, axis, gap, stretch)
script-err-set-params-stretch-is-true-false = set_params: stretch yra true / false
script-err-set-params-not-parametric-call-make = set_params: neparametrinis (pirmiau iškvieskite make_parametric)
script-err-paint-no-material-n-create-it = paint: nėra medžiagos { $n } (sukurkite ją su material(name, colour))
script-err-paint-expected-a-material-name-got = paint: tikėtasi medžiagos pavadinimo, gauta { $t }
script-err-show-hide-groups-and-components-only = show / hide: tik grupės ir komponentai
script-err-outer-shell-solids-must-be-in = outer_shell: kietieji kūnai turi būti viename kontekste
script-err-group-entities-must-be-in-one = group: objektai turi būti viename kontekste
script-err-scale-zero-factor = scale: nulinis koeficientas
script-err-rotate-zero-axis = rotate: nulinė ašis
script-err-follow-me-the-path-must-be = follow_me: trajektorija turi būti paviršiaus kontekste
script-err-note-not-added = note: nepridėta
script-err-dimension-not-added = dimension: nepridėta
script-err-what-sizes-must-be-positive-a = { $what }: dydžiai turi būti teigiami (toro vamzdis plonesnis už spindulį), segmentai 3..1000
script-err-cylinder-no-face = cylinder: nėra paviršiaus
script-err-cylinder-radius-and-height-must-be = cylinder: spindulys ir aukštis turi būti nenuliniai, segmentai 3..1000
script-err-box-sizes-must-be-non-zero = box: dydžiai turi būti nenuliniai
script-err-circle-radius-must-be-positive-segments = circle: spindulys turi būti teigiamas, segmentai 3..1000
script-err-face-points-are-not-a-valid = face: taškai nesudaro tinkamo plokščio daugiakampio
script-err-soft-not-an-edge = soft: ne briauna
script-err-faces-not-an-edge-or-face = faces: ne briauna ir ne paviršius
script-err-length-not-an-edge = length: ne briauna
script-err-time-limit-of-secs-s-exceeded = viršyta { $secs } s laiko riba
script-err-both-solids-must-be-in-the = abu kietieji kūnai turi būti tame pačiame kontekste
script-err-expected-a-colour-rrggbb-or-r = tikėtasi spalvos: "#RRGGBB" arba [r, g, b]
script-err-bad-colour-s-use-rrggbb = bloga spalva { $s } (naudokite #RRGGBB)
script-err-bad-colour-s = bloga spalva { $s }
script-err-curve-not-added = kreivė nepridėta
script-err-degenerate-curve = išsigimusi kreivė
script-err-expected-edges-got-e = tikėtasi briaunų, gauta { $e }
script-err-expected-a-group-or-component-got = tikėtasi grupės arba komponento, gauta { $e }
script-err-expected-a-face-got-e = tikėtasi paviršiaus, gauta { $e }
script-err-entity-no-longer-exists-merged-split = objekto nebėra (sujungtas, padalytas arba ištrintas)
script-err-context-no-longer-exists = konteksto nebėra
script-err-expected-one-entity-got-len = tikėtasi vieno objekto, gauta { $len }
script-err-expected-an-entity-or-an-array = tikėtasi objekto arba objektų masyvo, gauta { $type_name }
script-err-expected-an-entity-got-did-the = tikėtasi objekto, gauta () (ar operacija nepavyko?)
script-err-no-group-component-named-name-here = čia nėra grupės / komponento pavadinimu `{ $name }`
script-err-expected-an-array-of-points = tikėtasi taškų masyvo
script-err-expected-a-point-vec-or-x = tikėtasi taško (vec3 arba [x, y, z]), gauta { $type_name }
script-err-expected-a-number-got-type-name = tikėtasi skaičiaus, gauta { $type_name }
script-err-not-a-length-s = ne ilgis: { $s }
script-err-no-model = nėra modelio

## generator.rs
script-err-no-such-instance = tokio egzemplioriaus nėra
script-err-not-a-generated-component = ne sugeneruotas komponentas
script-err-no-such-context = tokio konteksto nėra
script-err-no-generator-name-in-the-document = dokumente nėra generatoriaus `{ $name }`
script-err-name-generators-nested-too-deep = { $name }: generatoriai įdėti per giliai
script-err-name-check-expr-is-not-true = { $name }: patikra `{ $expr }` nėra true / false
script-err-expected-a-number-got-t = tikėtasi skaičiaus, gauta { $t }

## json.rs
script-err-bad-entity-id-s = blogas objekto id { $s }

## rig.rs
script-err-part-has-no-parameter-key = `{ $part }` neturi parametro `{ $key }`
script-err-part-has-no-parameters = `{ $part }` neturi parametrų
script-err-no-part-part-in-the-component = komponente nėra dalies `{ $part }`
script-err-part-no-parameter-k-it-has = { $part }: nėra parametro `{ $k }` (yra { $keys })
script-err-part-the-part-has-no-parameters = { $part }: dalis neturi parametrų (parametrų turi tik sugeneruoti komponentai ir komponentai su parametrais)
script-err-part-show-hide-needs-a-yes = { $part }: show / hide reikia taip / ne arba skaičiaus parametro
script-err-part-choose-the-part-s-parameter = { $part }: pasirinkite dalies parametrą
script-err-part-the-direction-is-zero = { $part }: kryptis nulinė
script-err-part-the-axis-has-no-direction = { $part }: ašis neturi krypties
script-err-part-choose-a-parameter-or-write = { $part }: pasirinkite parametrą arba įrašykite išraišką
script-err-a-joint-without-a-part = jungtis be dalies
script-err-k-twice-parameters-and-formulas-share = `{ $k }` du kartus (parametrai ir formulės dalijasi pavadinimais)
script-err-no-such-component = tokio komponento nėra
script-err-this-component-is-built-by-a = šį komponentą sukuria generatorius arba parametrinė grupė
script-err-groups-can-t-have-parameters-make = grupės negali turėti parametrų: pirmiau paverskite ją komponentu
script-err-not-a-component-with-parameters = ne komponentas su parametrais
script-err-part-the-part-has-no-parameters-2 = { $part }: dalis neturi parametrų
script-err-part-no-such-part = { $part }: tokios dalies nėra
script-err-what-the-value-is-not-a = { $what }: reikšmė nėra skaičius, požymis ar tekstas
script-err-what-the-value-is-not-a-2 = { $what }: reikšmė nėra medžiagos pavadinimas
script-err-what-the-value-is-not-true = { $what }: reikšmė nėra true / false
script-err-what-the-value-is-not-a-3 = { $what }: reikšmė nėra skaičius

## ======== Branduolio pranešimai ========

## curves.rs
kernel-err-the-curve-bounds-faces-outside-its = kreivė riboja paviršius už savo plokštumos
kernel-err-other-edges-are-attached-to-the = prie kreivės prisegtos kitos briaunos
kernel-err-too-few-points = per mažai taškų
kernel-err-segments-must-be-min-max-curve = segmentų turi būti { $min }..{ $MAX_CURVE_SEGMENTS }
kernel-err-radius-must-be-positive = spindulys turi būti teigiamas
kernel-err-not-an-arc-circle-or-polygon = ne lankas, apskritimas ar daugiakampis

## chamfer.rs
kernel-err-the-edges-at-a-corner-are = kampo briaunos yra vienoje tiesėje
kernel-err-a-corner-of-the-edge-is = briaunos kampas nėra paprastas (per daug briaunų ar paviršių)
kernel-err-the-chamfer-is-too-big-for = nuožulna per didelė šiai briaunai
kernel-err-the-chamfer-is-too-big-for-2 = nuožulna per didelė gretimoms briaunoms
kernel-err-an-end-lies-along-the-edge = galas yra išilgai briaunos
kernel-err-the-radius-must-be-positive = spindulys turi būti teigiamas
kernel-err-the-distance-must-be-positive = atstumas turi būti teigiamas
kernel-err-the-faces-are-coplanar = paviršiai yra vienoje plokštumoje
kernel-err-broken-face-loop = sugadintas paviršiaus kontūras
kernel-err-the-edge-is-too-short = briauna per trumpa
kernel-err-the-edge-must-be-shared-by = briauna turi priklausyti lygiai dviem paviršiams

## follow_me.rs
kernel-err-path-reverses = trajektorija apsisuka atgal
kernel-err-profile-is-parallel-to-the-path = profilis lygiagretus trajektorijai
kernel-err-path-is-not-connected = trajektorija nėra vientisa
kernel-err-path-branches = trajektorija šakojasi
kernel-err-empty-path = tuščia trajektorija

## mod.rs
kernel-err-nothing-to-flip = nėra ką apversti
kernel-err-flip-axis-is-zero = apvertimo ašis nulinė

## push_pull.rs
kernel-err-through-hole-cap-vertex-missing = skylė skersai: trūksta dangtelio viršūnės
kernel-err-push-pull-would-collapse-geometry = stūmimas / traukimas sugriautų geometriją
kernel-err-face-loop-without-edge = paviršiaus kontūras be briaunos
kernel-err-distance-is-not-finite = atstumas nėra baigtinis

## params.rs
kernel-err-empty-base = tuščias pagrindas
kernel-err-size-must-be-positive = dydis turi būti teigiamas
kernel-err-not-parametric = neparametrinis
kernel-err-empty-group = tuščia grupė
kernel-err-already-parametric = jau parametrinis

## spline.rs
kernel-err-the-curve-would-cross-itself = kreivė kirstų pati save
kernel-err-segments-must-be-max-curve-segments = segmentų turi būti 1..{ $MAX_CURVE_SEGMENTS }
kernel-err-not-a-curve-that-can-be = tai ne redaguojama kreivė

## stats.rs
kernel-err-an-empty-name = tuščias pavadinimas

## text3d.rs
kernel-err-no-outlines = nėra kontūrų

## generator.rs
kernel-gen-name-no-parameter-k-join = { $name }: nėra parametro `{ $k }` ({ $join })
kernel-gen-empty-generator-name = tuščias @generator pavadinimas
kernel-gen-not-a-generator-the-header-needs = ne generatorius: antraštėje turi būti `// @generator Name`
kernel-gen-key-twice-formulas-and-parameters-share = `{ $key }` du kartus (formulės ir parametrai dalijasi pavadinimais)
kernel-gen-key-bool-default-is-true-false = { $key }: bool numatytoji reikšmė yra true / false
kernel-gen-unknown-parameter-type-t-length-int = nežinomas parametro tipas `{ $t }` (length, int, number, bool, choice, angle, percent, material)
kernel-gen-key-choice-needs-options-choice-a = { $key }: choice reikia parinkčių: choice(a|b|c)
kernel-gen-bad-parameter-name-key = blogas parametro pavadinimas `{ $key }`
kernel-gen-check-failed-expr = patikra nepavyko: { $expr }
kernel-gen-key-empty-formula = { $key }: tuščia formulė
kernel-gen-key-a-formula-is-length-int = { $key }: formulė yra length, int, number, bool, angle, percent arba text, o ne `{ $t }`
kernel-gen-bad-formula-name-key = blogas formulės pavadinimas `{ $key }`
kernel-gen-key-bad-number-s-in-the = { $key }: blogas skaičius `{ $s }` intervale
kernel-gen-key-not-a-name-s = { $key }: ne { $name }: { $s }
kernel-gen-key-s-is-not-one-of = { $key }: `{ $s }` nėra viena iš { $join }

## model.rs
kernel-err-the-generator-s-script-can-t = generatoriaus scenarijaus kopijuoti negalima
kernel-err-nothing-selected = nieko nepasirinkta

## solid.rs
kernel-err-pick-at-least-two-solids = pasirinkite bent du kietuosius kūnus
kernel-err-the-second-solid-has-a-degenerate = antrojo kietojo kūno transformacija išsigimusi
kernel-err-the-solids-do-not-overlap = kietieji kūnai nesidengia
kernel-err-pick-two-different-solids = pasirinkite du skirtingus kietuosius kūnus
# $name yra grupės / komponento pavadinimas
kernel-err-is-not-a-solid-every-edge = „{ $name }“ nėra kietasis kūnas (kiekviena briauna turi riboti lygiai du paviršius, jokių įdėtųjų grupių)

## OpError
kernel-err-entity-not-found = objektas nerastas
kernel-err-degenerate-geometry = išsigimusi geometrija
# $why yra priežastis
kernel-err-operation-not-possible = operacija neįmanoma: { $why }

## ======== Importas ir eksportas ========

## dae.rs
kernel-io-not-a-collada-document = ne COLLADA dokumentas
kernel-io-nodes-nested-too-deep = mazgai įdėti per giliai

## dxf.rs
kernel-io-no-supported-entities-line-polyline-dface = nėra palaikomų objektų (LINE, POLYLINE, 3DFACE, CIRCLE, ARC)
kernel-io-not-a-dxf-file-no-section = ne DXF failas (nėra SECTION)
kernel-io-not-a-dxf-file = ne DXF failas

## fbx.rs
kernel-io-fbx-description = FBX: { $description }

## gltf.rs
kernel-io-index-out-of-range = indeksas už ribų
kernel-io-no-meshes-in-the-gltf-scene = glTF scenoje nėra tinklų
kernel-io-node-hierarchy-too-deep = mazgų hierarchija per gili
kernel-io-draco-e = Draco: { $e }
kernel-io-draco-primitive-without-bufferview = Draco primityvas be bufferView
kernel-io-accessor-out-of-range = accessor už ribų
kernel-io-unsupported-component-type = nepalaikomas komponento tipas
kernel-io-unsupported-accessor-type = nepalaikomas accessor tipas
kernel-io-accessor-without-count = accessor be count
kernel-io-bufferview-out-of-range = bufferView už ribų
kernel-io-bufferview-without-bytelength = bufferView be byteLength
kernel-io-bad-buffer-index = blogas buferio indeksas
kernel-io-buffer-without-data = buferis be duomenų
kernel-io-compressed-gltf-e-is-not-supported = suspaustas glTF ({ $e }) nepalaikomas
kernel-io-only-gltf-is-supported = palaikomas tik glTF 2.0
kernel-io-external-file-uri-needs-a-path = išoriniam failui { $uri } reikia kelio
kernel-io-bad-base-in-data-uri = blogas base64 data URI
kernel-io-not-a-gltf-file-e = ne glTF failas: { $e }
kernel-io-glb-without-json = GLB be JSON
kernel-io-glb-json-e = GLB JSON: { $e }
kernel-io-truncated-glb-chunk = nutrauktas GLB blokas

## meshes.rs
kernel-io-no-faces-in-the-file = faile nėra paviršių

## native.rs
kernel-io-unsupported-version-version = nepalaikoma versija { $version }
kernel-io-not-a-varstotas-material-format-format = ne Varstotas medžiaga (format = { $format })
kernel-io-missing-root-definition = trūksta šakninės apibrėžties
kernel-io-not-a-varstotas-file-format-format = ne Varstotas failas (format = { $format })

## obj.rs
kernel-io-obj-line-ln-line = OBJ eilutė { $ln }: { $line }

## ply.rs
kernel-io-no-faces-a-point-cloud-can = nėra paviršių (taškų debesies importuoti negalima)
kernel-io-list-too-long = sąrašas per ilgas
kernel-io-unknown-format-f = nežinomas formatas { $f }
kernel-io-property-before-element = property prieš element
kernel-io-bad-element-count = blogas elementų skaičius
kernel-io-no-end-header = nėra end_header
kernel-io-not-a-ply-file = ne PLY failas
kernel-io-truncated-data = nutraukti duomenys
kernel-io-unknown-type-s = nežinomas tipas { $s }

## points.rs
kernel-io-no-points-in-display = nėra taškų { $display }

## stl.rs
kernel-io-stl-vertex-count-not-a-multiple = STL viršūnių skaičius nesidalija iš 3
kernel-io-bad-stl-vertex-line = bloga STL viršūnė: { $line }
kernel-io-not-a-valid-stl = netinkamas STL

## threemf.rs
kernel-io-mf-no-d-model-part = 3MF: nėra 3D modelio dalies
kernel-io-mf-no-object-id = 3MF: nėra objekto { $id }
kernel-io-mf-components-nested-too-deep = 3MF: komponentai įdėti per giliai
kernel-io-mf-no-part-path = 3MF: nėra dalies { $path }

## xml.rs
kernel-io-xml-no-root-element = XML: nėra šakninio elemento
kernel-io-xml-unbalanced-tags = XML: nesubalansuotos žymės
kernel-io-xml-e = XML: { $e }

## zip.rs
kernel-io-name-zip-method-m-not-supported = { $name }: zip metodas { $m } nepalaikomas
kernel-io-truncated-zip = nutrauktas zip
kernel-io-bad-zip-entry-name = blogas zip įrašas { $name }
kernel-io-bad-zip-directory = blogas zip katalogas

## COLLADA / PLY skaitytuvų priešdėliai
# $msg yra paties skaitytuvo pranešimas
kernel-io-collada = COLLADA: { $msg }
kernel-io-ply = PLY: { $msg }

## IoError
# $why yra sistemos klaida / skaitytuvo pranešimas
kernel-io-error-io = io: { $why }
kernel-io-error-format = format: { $why }

## File templates
template-files-heading = Iš šablonų failų
template-files-empty = Šiame aplanke nėra .vrst šablonų
template-open-folder = Atverti šablonų aplanką…
template-folder-error = Nepavyko perskaityti šablonų: { $error }
template-created = Sukurta iš šablono: { $path }
template-load-failed = Nepavyko įkelti šablono { $path }: { $error }
template-folder-label = Šablonų aplankas
template-folder-tip = Įkelkite čia .vrst modelius, kad galėtumėte pagal juos kurti naujus dokumentus.

app-export-replace-title = Pakeisti eksporto failus?
app-export-replace-files =
    Eksportuojant bus pakeisti šie medžiagų arba tekstūrų failai:
    { $files }
    Tęsti?
app-export-folder = Pasirinkite aplanką failui { $file } ir jo failams (medžiagoms, tekstūroms)
app-export-replace-all =
    Eksportuojant bus pakeisti šie failai:
    { $files }
    Tęsti?
app-import-missing-title = Trūksta failų šalia modelio
app-import-missing =
    { $file } nurodo šalia esančius failus, kurių Varstotas nepavyko perskaityti:
    { $files }
    Pasirinkti aplanką, kuriame yra modelis, kad jie būtų įkelti? (Smėlio dėžės programa, pvz., Flatpak, mato tik dialoge pasirinktą failą.)
app-import-not-in-folder = { $file } nėra aplanke { $folder }

## Object and tag locking
cmd-lock = Užrakinti
cmd-unlock = Atrakinti
cmd-unlock-all = Atrakinti viską
cmd-desc-lock = Užrakinti pasirinktus objektus, kad jų nebūtų galima pasirinkti ar redaguoti
cmd-desc-unlock-all = Atrakinti visus objektus ir žymas
edit-locked = Šis veiksmas pakeistų užrakintus objektus. Pirmiausia juos atrakinkite.
script-member-locked = Veiksmingas redagavimo užraktas, įskaitant žymos ir tėvinių objektų užraktus (tik skaityti).

## Pradžios ekranas ir atsiliepimai
start-menu = Pradžios ekranas
start-subtitle = Kurkite modelį, tęskite darbą arba peržiūrėkite pavyzdžius.
start-create = Naujas modelis
start-new = Kurti
start-open = Atverti dokumentą…
start-guide = Trumpasis vadovas
start-recent = Naujausi dokumentai
start-no-recent = Čia bus rodomi neseniai atverti dokumentai.
start-basics-title = Pirmieji žingsniai
start-basics = Tarpas: žymėjimas · L: linija · R: stačiakampis. Tikslius matmenis įveskite matmenų laukelyje. Pelės ratukas keičia mastelį, vidurinis mygtukas suka vaizdą.
start-show = Rodyti paleidžiant
start-continue = Tęsti
report-menu = Pranešti apie problemą…
report-intro = Peržiūrėkite pranešimą ir atverkite juodraštį naršyklėje. Norėdami išsiųsti, prisijunkite prie GitHub ir pateikite jį patys.
report-access = Privačiai saugyklai reikia prieigos. Modelio failai ir ekrano nuotraukos automatiškai nepridedami.
report-title = Santrauka
report-description = Kas nutiko?
report-description-hint = Atkūrimo žingsniai, tikėtinas rezultatas ir faktinis elgesys.
report-errors = Pridėti paskutinius 10 įspėjimų ir klaidų
report-review = Prieš bendrindami peržiūrėkite tekstą. Žurnale gali būti failų vardų ir kitos informacijos.
report-copy = Kopijuoti pranešimą
report-github = Atverti GitHub juodraštį
report-paste = Visas pranešimas nukopijuotas. Įklijuokite jį į GitHub problemos aprašymą.

cli-usage =
    Naudojimas: Varstotas [--open] <document.vrst>
    Varstotas -- <document.vrst>
    Varstotas --help | --version
cli-one-document = Nurodykite vieną dokumentą
cli-unknown-option = Nežinomas parametras: { $option }
cli-open-path = Po --open reikia nurodyti dokumento kelią

prefs-save-backups = Saugoti ankstesnės išsaugotos versijos atsarginę kopiją
prefs-save-backups-note = Išsaugant ankstesnė versija paliekama { DATA_DIR() }/backups kaip <name> (…).bak.vrst. Išjungus esamos kopijos nepašalinamos.

model-info-metadata = Dokumento metaduomenys
model-info-metadata-text = Tekstas / aprašymas
model-info-license = Licencija
model-info-license-hint = Licencijos pavadinimas, nuoroda arba visas tekstas (neprivaloma)
model-info-metadata-apply = Taikyti
model-info-metadata-reset = Atmesti pakeitimus
model-info-format-version = Išsaugojimo formato versija: { $version }
model-info-metadata-edit = Keisti dokumento metaduomenis

model-info-tab-info = Informacija
model-info-tab-meta = Metaduomenys
model-info-tab-cleanup = Valymas

download-progress-unknown = Atsisiųsta { $received } / bendras dydis nežinomas

# Practical step-by-step walkthrough.
tutorial-title = Pamoka
tutorial-step = Žingsnis { $current } / { $total }
tutorial-try = Išbandykite
tutorial-tools = Įjungti įrankį
tutorial-details = Daugiau apie šį žingsnį
tutorial-practice-note = Palikite šį langą šalia modelio ir praktikuokitės darbo srityje.
tutorial-back = Atgal
tutorial-restart = Nuo pradžių
tutorial-finish = Baigti
tutorial-next = Toliau
tutorial-0-title = Pažinkite Varstotas
tutorial-0-body = Modelį sudaro briaunos ir paviršiai, jungiami į grupes bei pakartotinai naudojamus komponentus. Programa turi tikslų braižymą, keičiamas 3D formas, medžiagas, scenas ir brėžinių lapus. Pradėsime nuo naujo dokumento.
tutorial-0-task = Pasirinkite File → New. Paprasto stalviršio pavyzdžiu išmoksite įrankius ir paruošite failą dalytis. Toliau eikite savo tempu.
tutorial-1-title = Apžiūrėkite modelį
tutorial-1-body = Tempkite viduriniu pelės mygtuku, kad pasuktumėte vaizdą; su Shift — pastumtumėte. Ratukas artina ties žymekliu. Jutiklinio kilimėlio valdymas ir greitis keičiami navigacijos nustatymuose.
tutorial-1-task = Išbandykite sukimą ir stumdymą. Camera → Standard Views perjunkite Top ir Iso, tada naudokite Zoom Extents.
tutorial-2-title = Nubrėžkite tikslų pagrindą
tutorial-2-body = Uždaras plokščias briaunų kontūras sudaro paviršių. Rectangle ir Circle sukuria jį iškart. Measurements laukas priima tikslius matmenis, kol įrankis aktyvus; Patarimai paaiškina dabartinį etapą.
tutorial-2-task = Įjunkite Rectangle. Nurodykite pirmą kampą, pajudinkite žymeklį, į Measurements įrašykite 1400mm,800mm ir spauskite Enter.
tutorial-3-title = Suteikite tūrį
tutorial-3-body = Push/Pull ištempia paviršių. Draw → Shapes viena operacija kuria dėžę, cilindrą, kūgį, piramidę, sferą ar torą. Šie komponentai turi keičiamus generatoriaus parametrus Savybės.
tutorial-3-task = Ištempkite stačiakampį su Push/Pull ir įveskite 40. Šalia išbandykite Box: pirmas kampas, priešingas pagrindo kampas ir aukštis.
tutorial-4-title = Naudokite pritraukimą
tutorial-4-body = Žymeklis atpažįsta galus, vidurio taškus, ašis ir tinklelį. Spalvotos linijos padeda išlaikyti kryptį. Rodyklės fiksuoja ašį, Shift — dabartinę kryptį. Magneto meniu valdo pritraukimo rūšis.
tutorial-4-task = Įjunkite tinklelį ir Line. Nuo kampo judėkite ašies kryptimi iki kitos tinklelio linijos. Pakartokite su Box pagrindu ir aukščiu. Escape atšaukia nebaigtą veiksmą.
tutorial-5-title = Redaguokite ir kartokite
tutorial-5-body = Move, Rotate ir Scale keičia pažymėtus objektus; Undo leidžia eksperimentuoti. Grupė atskiria geometriją nuo gretimų briaunų. Komponento kopijos dalijasi apibrėžimu, todėl pakeitimai atnaujina visas kopijas.
tutorial-5-task = Pažymėkite stalviršį ir Edit meniu sukurkite grupę. Perkelkite ją, atšaukite perkėlimą ir dukart spustelėkite, kad redaguotumėte vidų. Baigę išeikite iš grupės.
tutorial-6-title = Sutvarkykite modelį
tutorial-6-body = Outliner rodo įdėtas grupes ir komponentus. Žymos valdo matomumą ir redagavimo užraktus. Savybės pateikia pažymėto objekto parametrus. Paslėpimas ir užrakinimas veikia atskirai.
tutorial-6-task = Savybės pavadinkite stalviršį. Tags skiltyje sukurkite ir priskirkite žymą, perjunkite matomumą. Išbandykite užraktą ir prieš dažydami atrakinkite.
tutorial-7-title = Medžiagos ir vaizdo stiliai
tutorial-7-body = Paint priskiria spalvą ir tekstūrą, Texture Position keičia jų padėtį. Online Materials atsiunčia medžiagas. Shaded with Textures tinka modeliuoti, Monochrome skiria paviršių puses, Realistic naudoja fizines savybes ir apšvietimą.
tutorial-7-task = Nudažykite paviršių. Palyginkite Shaded with Textures, Monochrome ir Realistic stilius. Įjunkite X-Ray paslėptoms briaunoms pamatyti.
tutorial-8-title = Pristatykite rezultatą
tutorial-8-body = Scenos įsimena vaizdus ir rodymo nustatymus. Matmenys pažymi modelį, pjūviai rodo vidų, lapai išdėsto vaizdus popieriuje. Render kuria galutinį atvaizdą, animacija pereina tarp scenų.
tutorial-8-task = Pridėkite Iso sceną ir antrą Top sceną. Persijunkite tarp jų, tada išbandykite Dimension arba Section.
tutorial-9-title = Išsaugokite ir dalykitės
tutorial-9-body = .vrst išlaiko redaguojamą geometriją, medžiagas, scenas ir metaduomenis. Dokumento informacija → Meta saugo aprašymą, licenciją ir peržiūrą. Importas ir eksportas palaiko OBJ, STL, GLB ir kitus formatus; scenarijai ir papildiniai plečia įrankius.
tutorial-9-task = Išsaugokite modelį. Dokumento informacija → Meta įrašykite aprašymą ir licenciją, vėl išsaugokite. Help → Examples rasite pavyzdžius; galite pakartoti bet kurį žingsnį.

app-reversed-faces = Apversta paviršių: { $count }. Monochrome režimas parodo priekinę ir galinę puses.

download-progress-previews = Atsisiunčiamos peržiūros: { $count }

prefs-trackpad-enabled = Navigacija jutikliniu kilimėliu / sklandžiu slinkimu
prefs-gesture-speed = Jautrumas
prefs-invert-swipe = Apversti dviejų pirštų slinkimą
prefs-pinch-enabled = Artinti suspaudimo gestu
prefs-invert-pinch = Apversti suspaudimą
prefs-twist = Dviejų pirštų pasukimas
prefs-twist-off = Išjungta
prefs-twist-orbit = Sukti apie vertikalią ašį
prefs-twist-roll = Pakreipti vaizdą
prefs-invert-twist = Apversti pasukimą
prefs-trackpad-platform = Suspaudimas ir pasukimas priklauso nuo OS ir tvarkyklės. Šie nustatymai taip pat taikomi sklandaus slinkimo įrenginiams.

view-triangles = Trikampiai
view-triangles-hint = Rodyti visas trianguliacijos briaunas virš esamo stiliaus, įskaitant minkštas briaunas ir daugiakampių įstrižaines.

# 3D Printing: basic surface checks and conservative repair.
menu-3d-printing = 3D Printing
print-title = Paruošimas 3D spausdinimui
print-check-command = Tikrinti modelį…
print-repair-command = Taisyti tinklą…
print-check-description = Tikrinti pasirinktus objektus arba matomą modelį dėl pagrindinių spausdinimo tinklo defektų.
print-repair-description = Taisyti siūles, dublikatus, orientaciją ir mažas plokščias angas.
print-scope = Sritis:
print-visible = Matomas modelis
print-selection = Pasirinkimas
print-tab-check = Patikra
print-tab-repair = Taisymas
print-working = Apdorojama modelio kopija…
print-worker-failed = Nepavyko baigti patikros. Paleiskite ją dar kartą.
print-result-stale = Modelis arba pasirinkimas pasikeitė. Taisymas nepritaikytas; pakartokite patikrą.
print-repair-undo = Taisymas 3D spausdinimui
print-stale = Ataskaita pasenusi. Pakartokite patikrą.
print-repair-weld = Sujungti sutampančias viršūnes
print-repair-degenerate = Šalinti išsigimusias sieneles
print-repair-duplicates = Šalinti pasikartojančias sieneles
print-repair-orient = Suvienodinti sienelių orientaciją
print-repair-shells = Orientuoti uždarus apvalkalus, išsaugant ertmes
print-repair-holes = Užverti mažas plokščias angas
print-hole-size = Didžiausias angos dydis:
print-repair-note = Taisymą galima atšaukti. Užrakinta, paslėpta ir procedūrinė geometrija išsaugoma. Sudėtingi ir įdėtiniai angų kontūrai paliekami rankiniam taisymui.
print-repair-whole-objects = Taisymui pasirinkite visas grupes / komponentus arba „Matomas modelis“.
print-repair-run = Taisyti ir tikrinti
print-repaired = Sujungta viršūnių: { $welded }; pašalinta sienelių: { $removed }; apversta sienelių: { $reversed }; užverta angų: { $holes }; praleista objektų: { $skipped }.
print-empty = Šioje srityje nėra matomų sienelių. Pasirinkite objektą arba „Matomas modelis“.
print-counts = Sienelių: { $faces } · trikampių: { $triangles } · apvalkalų: { $shells } (uždarų: { $closed })
print-size = Dydis: { $x } × { $y } × { $z } mm
print-area = Paviršiaus plotas: { $area } mm²
print-volume = Uždaras tūris: { $volume } mm³
print-found-problems = Rasta paviršiaus problemų
print-basic-pass = Pagrindinė paviršiaus patikra sėkminga
print-limitations = Sienelių storis vertinamas mėginiais. Sankirtos, iškyšos ir spausdintuvo ribos pateikiamos gretimose kortelėse. Patikrinkite sluoksnius pjaustyklėje.
print-tolerance = Viršūnių sutapimo tolerancija:
print-highlight = Paryškinti modelio problemas
print-check-run = Tikrinti
print-copy = Kopijuoti ataskaitą
print-open-edges = Atviros kraštinės
print-nonmanifold = Briaunos, turinčios daugiau nei dvi sieneles
print-winding = Nesuderinta briaunų orientacija
print-degenerate = Išsigimę trikampiai / tuščios sienelės
print-duplicates = Pasikartojantys trikampiai
print-inverted = Apversti uždari apvalkalai
print-wire = Atskiros briaunos (nespausdinamos)
print-export-visible = Eksportuoti matomą modelį

# Printing preparation tools
print-analysis = Analizė
print-bed = Spausdintuvas ir stalas
print-split = Pjūviai ir jungtys
print-export = Eksportas spausdinimui
print-export-stale = Modelis arba nuostatos pasikeitė. Prieš atverdami pjaustyklę eksportuokite iš naujo.
print-cut-undo = Pjūvis spausdinimui
print-place-undo = Išdėstymas ant spausdinimo stalo
print-placement-done = Išdėstymas atnaujintas. Pakartokite patikrą.
print-cut-select = Pjūviui pasirinkite vieną visą grupę arba komponentą.
print-cut-done = Sukurta dalių: { $parts }, įskaitant atskirus kaiščius: { $dowels }. Išdėstykite jas ant stalo.
print-export-extension = Pasirinkite .3mf arba .stl.
print-export-done = Eksportuota dalių: { $parts } į { $path }
print-slicer-opened = Eksportuotas failas atvertas numatytąja jam programa.
print-min-wall = Mažiausias sienelės storis
print-two-nozzles = Du purkštuko skersmenys
print-sample-budget = Didžiausias storio mėginių skaičius
print-pair-budget = Didžiausias tikrinamų sankirtų porų skaičius
print-overhang-angle = Iškyšos kampas nuo vertikalės žemyn
print-map = Paviršiaus žemėlapis
print-map-none = Nėra
print-map-wall = Sienelių storis
print-map-overhang = Iškyšos
print-intersection-count = Susikertančių trikampių porų: { $n }; patikrinta porų: { $pairs }
print-intersection-incomplete = Pasiektas porų limitas: sankirtų analizė nebaigta.
print-wall-coverage = Storio mėginių: { $samples } / { $possible }; trikampių: { $triangles } / { $total }
print-wall-result = Mažiausias išmatuotas storis: { $minimum } mm; plonų: { $thin }; nežinomų: { $unknown }
print-overhang-result = Iškyšų plotas: { $area } mm² ({ $n } trikampių)
print-wall-unreliable = Atviri, persidengiantys ar neteisingai orientuoti paviršiai mažina storio matavimo patikimumą.
print-wall-estimate = Storis vertinamas spinduliais į vidų, iki keturių mėginių trikampiui. Smulkios detalės tarp mėginių gali likti nepastebėtos. Pilka reiškia, kad išėjimas nerastas; raudona — storis mažesnis už ribą. Sąlytis su stalu nepriskiriamas iškyšoms. Patikrinkite sluoksnius pjaustyklėje.
print-profile-name = Spausdintuvo profilis
print-bed-width = Stalo plotis X
print-bed-depth = Stalo gylis Y
print-bed-height = Spausdinimo aukštis Z
print-bed-margin = Atstumas nuo stalo krašto
print-bed-gap = Tarpas tarp dalių
print-nozzle = Purkštuko skersmuo
print-arrange-rotate = Leisti pasukti 90° išdėstant
print-bed-show = Rodyti spausdinimo tūrį scenoje
print-drop = Nuleisti ant stalo
print-center = Centruoti ant stalo
print-arrange = Išdėstyti dalis
print-lay-face = Paguldyti pasirinkta siena
print-placement-note = Išdėstymas išlaiko mastelį ir naudoja visas dalis. Dėstoma pagal ribojančius stačiakampius; sudėtingas formas gali tekti išdėstyti ranka. Persidengimo įspėjimai taikomi ribojantiems stačiakampiams.
print-bed-result = Dalių: { $parts }; už ribų: { $outside }; ne ant stalo: { $floating }; ribojančių stačiakampių persidengimų: { $overlaps }
print-fits = Telpa
print-outside = Už spausdinimo tūrio ribų
print-cut-normal = Pjūvio plokštumos normalė
print-cut-offset = Plokštumos poslinkis nuo koordinačių pradžios
print-connectors = Pridėti lizdus ir atskirus kaiščius
print-dowel-count = Kaiščių skaičius
print-dowel-diameter = Kaiščio skersmuo
print-dowel-length = Kaiščio ilgis
print-dowel-clearance = Radialinis ir ašinis tarpelis
print-dowel-wall = Mažiausia sienelė aplink lizdą
print-dowel-note = Pjūviams reikia uždarų, nuosekliai orientuotų dalių. Lizdai išdėstomi išilgai pjūvio, tikrinant aplinkinę medžiagą. Pusės ir atskiri kaiščiai lieka surinkimo vietose; vėliau juos išdėstykite. Tarpelį pritaikykite savo spausdintuvui.
print-cut-run = Perpjauti ir uždaryti pjūvius
print-cut-to-bed = Skaidyti pagal spausdintuvo matmenis
print-cut-to-bed-note = Automatiniai pjūviai naudoja naudingus X/Y/Z matmenis, iki 256 dalių. Dalys uždaromos be jungčių ir paliekamos surinkimo vietose.
print-export-scope-note = Eksportuojama pasirinkta sritis, išlaikant pasaulio koordinates milimetrais. 3MF išlaiko atskirus objektus; STL sudaro trikampiai.
print-export-materials-note = 3MF įtraukia bazines spalvas, pavadinimus, aprašą, licenciją ir šaltinius. PBR tekstūros neeksportuojamos. STL turi tik geometriją.
print-export-3mf = Eksportuoti 3MF…
print-export-stl = Eksportuoti STL…
print-slicer-open = Atverti pjaustyklėje
print-slicer-note = Eksportas atveriamas sistemos numatytąja .3mf / .stl programa: nustatykite pjaustyklę numatytąja šiems failams. Pakeitę modelį eksportuokite iš naujo.
print-export-missing = Eksportuoto failo nebėra. Eksportuokite iš naujo.

# Printing operation errors
print-error-1 = Netinkama pirminio objekto transformacija
print-error-2 = Šioje srityje nėra matomų spausdinamų sienų
print-error-3 = Netinkami spausdintuvo matmenys arba paraštė
print-error-4 = Užrakintų ir procedūrinių objektų keisti negalima
print-error-5 = Dalyje yra užrakintos geometrijos
print-error-6 = Prieš pjaudami parodykite visą dalies geometriją
print-error-7 = Dalyje yra užrakintų briaunų
print-error-8 = Dalyje yra užrakintų objektų
print-error-9 = Pasirinkite visas dalis
print-error-10 = Išdėstymui ir pjūviams pasirinkite visas grupes arba komponentus
print-error-11 = Prieš išdėstymą sugrupuokite laisvą geometriją, jei kitos šakninės sienos paslėptos
print-error-12 = Nėra matomų dalių
print-error-13 = Dalis neturi paviršiaus
print-error-14 = Negrįžtama pirminio objekto transformacija
print-error-15 = Netinkami spausdintuvo matmenys
print-error-16 = Nėra dalių
print-error-17 = Pradiniu masteliu dalys netelpa ant vieno stalo: pasirinkite mažiau dalių arba jas perpjaukite
print-error-18 = Išsigimusi siena
print-error-19 = Pasirinkta siena įgilinta: dalis geometrijos atsidurtų po stalu
print-error-20 = Nevienareikšmis pjūvio kontūras: prieš pjaudami pataisykite dalį
print-error-21 = Atviras pjūvio kontūras
print-error-22 = Netinkamas pjūvio kontūras
print-error-23 = Pjūvio angai nerastas išorinis kontūras
print-error-24 = Uždarius pjūvį paviršius liko atviras: pirmiausia pataisykite pradinį tinklą
print-error-25 = Plokštuma nepadalija šios dalies
print-error-26 = Netinkama pjūvio plokštuma
print-error-27 = Pjūviui pasirinkite tiksliai vieną visą dalį
print-error-28 = Prieš pjaudami pataisykite atvirus, pasikartojančius ir nenuosekliai orientuotus paviršius
print-error-29 = Pjūviui reikia baigtos sankirtų patikros be susikertančių paviršių
print-error-30 = Pasirinkite vieną visą dalį
print-error-31 = Reikėtų daugiau nei 256 dalių: padidinkite profilį arba pjaukite etapais
print-error-32 = Netinkami kaiščių matmenys, skaičius arba tarpelis
print-error-33 = Jungtims nerastas pjūvio paviršius
print-error-34 = Jungtys per arti: sumažinkite jų skaičių arba skersmenį
print-error-35 = Jungtis išeitų už dalies ribų arba nepakanka sienelės/gylio. Sumažinkite dydį ar skaičių, pakeiskite plokštumą arba pasirinkite storesnę sritį
print-error-36 = Netinkamas lizdas
print-error-37 = Nepavyko išpjauti lizdo
print-error-38 = Netinkamas kaištis
print-bed-frame = Aprėpti spausdinimo tūrį
print-part-name = Spausdinama dalis
print-socket-name = Lizdo pjoviklis
print-dowel-name = Sujungimo kaištis

print-go-check = Eiti į patikrą
print-analysis-run = Vykdyti analizę
print-bed-refresh = Atnaujinti stalo patikrą

# Window → Physics
physics-title = Fizika
physics-drop = Kritimas
physics-drop-hint = Pažymėtos grupės ir komponentai nukrenta ir nurimsta ant visko, kas matoma.
physics-shape = Forma
physics-shape-hull = Apvalkalas
physics-shape-hull-hint = Kiekvieno objekto iškilasis apvalkalas: greita ir stabilu, įdubos užpildomos.
physics-shape-pieces = Dalys
physics-shape-pieces-hint = Iškilosios kiekvieno objekto dalys: į dubenį galima kažką įdėti, ruošiama ilgiau.
physics-friction = Trintis
physics-bounce = Atšokimas
physics-gravity = Gravitacija
physics-time = Laiko riba
physics-floor = Grindys ties Z = 0
physics-no-bodies = Pažymėkite grupes ar komponentus, kurie turi nukristi.
physics-bodies = Krentančių objektų: { $n }
physics-run = Paleisti
physics-running = Modeliuojama… { $time } s
physics-done = Nurimo per { $time } s
physics-stopped = Sustabdyta ties { $time } s
physics-fell = Objektų be atramos: { $n } (jie lieka vietoje)
physics-apply = Taikyti
physics-again = Paleisti iš naujo
physics-stop = Stabdyti
physics-cancel = Atšaukti
physics-interrupted = Modelis pasikeitė: modeliavimas sustabdytas.
physics-failed = Modeliavimas nepavyko: { $error }
physics-undo = Kritimas
physics-applied = Perkelta objektų: { $n }
physics-cloth = Audinys
physics-cloth-hint = Pažymėta siena (arba grupė iš vienos sienos) krenta kaip audinys ir apgaubia visa kita, kas matoma. Ji tampa lygių trikampių grupe.
physics-cloth-no-face = Pažymėkite vieną sieną arba grupę iš vienos sienos.
physics-cloth-too-small = Siena mažesnė už žingsnį.
physics-cloth-too-fine = Taškų: { $n } — padidinkite žingsnį.
physics-cloth-particles = Audinio taškų: { $n }
physics-cloth-spacing = Žingsnis
physics-cloth-spacing-hint = Atstumas tarp audinio taškų: kuo mažesnis, tuo smulkesnės klostės ir ilgesnis skaičiavimas.
physics-cloth-stiffness = Standumas
physics-cloth-stiffness-hint = Kiek audinys priešinasi tempimui.
physics-cloth-bending = Lenkimas
physics-cloth-bending-hint = Kiek audinys priešinasi lenkimui: mažai — šilkas, daug — oda.
physics-cloth-thickness = Storis
physics-cloth-thickness-hint = Tarpas tarp audinio ir to, ant ko jis guli (ne mažiau nei dešimtadalis žingsnio).
physics-cloth-self = Nepraeina pro save
physics-cloth-self-hint = Klostės susiduria tarpusavyje (lėčiau).
physics-cloth-undo = Audinys
physics-cloth-applied = Audinys tapo nauja grupe.
