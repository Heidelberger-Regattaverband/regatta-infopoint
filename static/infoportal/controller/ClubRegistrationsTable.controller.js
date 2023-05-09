sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "../model/Formatter"
], function (BaseController, JSONModel, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ClubRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oTable = this.getView().byId("registrationsTable");

      this._oRegistrationsModel = new JSONModel();
      this.setViewModel(this._oRegistrationsModel, "registrations");

      this.getRouter().getRoute("clubRegistrations").attachPatternMatched(this._onPatternMatched, this);
    },

    onNavBack: function () {
      this.navBack("participatingClubs");
    },

    _onPatternMatched: async function (oEvent) {
      const clubId = oEvent.getParameter("arguments").clubId;
      const iRegattaId = this.getRegattaId();
      await this.updateJSONModel(this._oRegistrationsModel, `/api/regattas/${iRegattaId}/clubs/${clubId}/registrations`, this._oTable);
    }

  });
});