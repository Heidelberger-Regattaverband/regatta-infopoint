sap.ui.define([
  "de/regatta_hd/infopoint/controller/Base.controller",
  "sap/ui/model/json/JSONModel",
  "sap/m/MessageToast",
  "../model/Formatter"
], function (BaseController, JSONModel, MessageToast, Formatter) {
  "use strict";

  return BaseController.extend("de.regatta_hd.infopoint.controller.ClubRegistrationsTable", {

    formatter: Formatter,

    onInit: function () {
      this.getView().addStyleClass(this.getOwnerComponent().getContentDensityClass());

      this._oTable = this.getView().byId("registrationsTable");

      this._oRegistrationsModel = new JSONModel();
      this.setViewModel(this._oRegistrationsModel, "registrations");

      this.getRouter().getRoute("clubRegistrations").attachPatternMatched(async (oEvent) => await this._onPatternMatched(oEvent), this);
    },

    onNavBack: function () {
      this.navBack("participatingClubs");
    },

    onRefreshButtonPress: async function (oEvent) {
      await this._loadRegistrationsModel();
      MessageToast.show(this.i18n("msg.dataUpdated", undefined));
    },

    _onPatternMatched: async function (oEvent) {
      this._iClubId = oEvent.getParameter("arguments").clubId;
      await this._loadRegistrationsModel();
    },

    _loadRegistrationsModel: async function () {
      await this.updateJSONModel(this._oRegistrationsModel, `/api/regattas/${this.getRegattaId()}/clubs/${this._iClubId}/registrations`, this._oTable);
    }

  });
});