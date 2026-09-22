-- pgls-format: layout=expanded, commaStyle=leading, indentStyle=tabs, indentSize=4, lineWidth=80
INSERT INTO login_target.compte_bancaire (
    id_compte_bancaire,
    id_personne,
    ref_personne,
    titulaire_du_compte_bancaire,
    domiciliation_bancaire,
    iban,
    bic,
    rib
)
SELECT
    id_compte_bancaire,
    id_personne,
    ref_personne,
    titulaire_du_compte_bancaire,
    domiciliation_bancaire,
    iban,
    bic,
    rib
FROM login_linking.compte_bancaire;
